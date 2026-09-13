//! 仿真推进节奏的量化指标（联机卡顿的"数字眼"）。
//!
//! 背景：网络模式下 `simulation_driver_system` 在 `is_tick_ready` 为假时 `break`，
//! 但累加器**继续增长**；relay 定稿帧一到，同一帧内 `while` 会把积压的 tick 连续跑完。
//! 画面上就是"定格 → 跳变"（联机卡顿）；单机模式该分支永不触发，所以单机顺滑。
//!
//! 本模块把该现象变成可读数字（每帧推进了几个 tick、多少帧被阻塞、累加器峰值），
//! 供 BRP 探针读取 —— **自动化测试与人工验收共用同一组指标**，避免"感觉卡/不卡"的争论。

use bevy::prelude::*;

use crate::driver::SimulationDriver;

/// 一帧内推进 ≥ 该 tick 数视为"跳变"（可见的追赶）。
pub const BURST_THRESHOLD: u32 = 3;
/// 直方图分格：`0..=7` 各占一格，`8+` 归入最后一格。
pub const HISTOGRAM_BUCKETS: usize = 9;

/// 推进节奏的累计统计（进程内累加，随时可由探针读取快照）。
#[derive(Resource, Debug, Clone)]
pub struct PacingMetrics {
    /// 观测到的帧数。
    pub frames: u64,
    /// 累计推进的 tick 数。
    pub ticks_total: u64,
    /// 单帧推进 tick 数的最大值（跳变幅度的上界）。
    pub max_ticks_frame: u32,
    /// 本帧一个 tick 都没推进的帧数（"定格"的候选）。
    pub frames_zero: u64,
    /// 本帧推进 ≥ [`BURST_THRESHOLD`] 个 tick 的帧数（"跳变"的候选）。
    pub frames_burst: u64,
    /// 因等待远端定稿帧而中断推进的帧数（网络模式特有）。
    pub frames_blocked: u64,
    /// 每帧推进 tick 数的直方图，下标即 tick 数（最后一格为 8+）。
    pub histogram: [u64; HISTOGRAM_BUCKETS],
    /// 累加器峰值（秒）——大于 `tick_duration` 说明曾积压。
    pub accumulator_peak: f32,
}

impl Default for PacingMetrics {
    fn default() -> Self {
        Self {
            frames: 0,
            ticks_total: 0,
            max_ticks_frame: 0,
            frames_zero: 0,
            frames_burst: 0,
            frames_blocked: 0,
            histogram: [0; HISTOGRAM_BUCKETS],
            accumulator_peak: 0.0,
        }
    }
}

impl PacingMetrics {
    /// 记录一帧的推进结果：本帧推进了 `ticks` 个 tick，是否因等待远端帧中断，
    /// 以及本帧结束时的累加器值（秒）。
    pub fn observe_frame(&mut self, ticks: u32, blocked: bool, accumulator: f32) {
        self.frames += 1;
        self.ticks_total += u64::from(ticks);
        self.max_ticks_frame = self.max_ticks_frame.max(ticks);
        if ticks == 0 {
            self.frames_zero += 1;
        }
        if ticks >= BURST_THRESHOLD {
            self.frames_burst += 1;
        }
        if blocked {
            self.frames_blocked += 1;
        }
        let bucket = (ticks as usize).min(HISTOGRAM_BUCKETS - 1);
        self.histogram[bucket] += 1;
        if accumulator > self.accumulator_peak {
            self.accumulator_peak = accumulator;
        }
    }

    /// 平均每帧推进的 tick 数。稳定态应约等于 `tick 频率 / 帧率`。
    pub fn ticks_per_frame_avg(&self) -> f32 {
        if self.frames == 0 {
            return 0.0;
        }
        self.ticks_total as f32 / self.frames as f32
    }

    /// 阻塞帧占比：越高越容易出现"定格"。
    pub fn blocked_ratio(&self) -> f32 {
        if self.frames == 0 {
            return 0.0;
        }
        self.frames_blocked as f32 / self.frames as f32
    }

    /// 跳变帧占比：越高越容易出现"追赶式跳变"。
    pub fn burst_ratio(&self) -> f32 {
        if self.frames == 0 {
            return 0.0;
        }
        self.frames_burst as f32 / self.frames as f32
    }

    /// 清零，便于测量窗口开始前重置。
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// 每帧观测 `SimulationDriver` 的推进结果并累计。
///
/// 必须排在 `simulation_driver_system` **之后**，否则读到的是上一帧的字段。
pub fn pacing_metrics_system(driver: Res<SimulationDriver>, mut metrics: ResMut<PacingMetrics>) {
    metrics.observe_frame(
        driver.last_frame_ticks,
        driver.last_frame_blocked,
        driver.clock.accumulator,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 稳定推进（每帧 1 tick）不得出现定格或跳变。
    #[test]
    fn test_steady_stream_has_no_stall_or_burst() {
        let mut m = PacingMetrics::default();
        for _ in 0..60 {
            m.observe_frame(1, false, 0.01);
        }
        assert_eq!(m.frames, 60);
        assert_eq!(m.ticks_total, 60);
        assert_eq!(m.max_ticks_frame, 1);
        assert_eq!(m.frames_zero, 0, "稳定推进不应有定格帧");
        assert_eq!(m.frames_burst, 0, "稳定推进不应有跳变帧");
        assert_eq!(m.frames_blocked, 0, "稳定推进不应被阻塞");
        assert_eq!(m.histogram[1], 60);
        assert!((m.ticks_per_frame_avg() - 1.0).abs() < f32::EPSILON);
    }

    /// 经典的"等待→爆发"到达模式必须被量化出来：
    /// 两帧等待（0 tick + blocked），随后一帧补齐 3 个 tick。
    #[test]
    fn test_wait_then_burst_is_quantified() {
        let mut m = PacingMetrics::default();
        for _ in 0..10 {
            m.observe_frame(0, true, 0.10); // 等远端帧：本帧不推进，累加器积压
            m.observe_frame(0, true, 0.15);
            m.observe_frame(3, false, 0.02); // 帧到达：一帧补齐
        }
        assert_eq!(m.frames, 30);
        assert_eq!(m.frames_zero, 20);
        assert_eq!(m.frames_burst, 10);
        assert_eq!(m.frames_blocked, 20);
        assert_eq!(m.max_ticks_frame, 3);
        assert_eq!(m.histogram[0], 20);
        assert_eq!(m.histogram[3], 10);
        assert!((m.blocked_ratio() - 20.0 / 30.0).abs() < 1e-6);
        assert!((m.burst_ratio() - 10.0 / 30.0).abs() < 1e-6);
        assert!(
            (m.accumulator_peak - 0.15).abs() < 1e-6,
            "累加器峰值应被记录"
        );
    }

    /// 直方图最后一格收纳 8+，且 reset 能清零（测量窗口需要）。
    #[test]
    fn test_histogram_overflow_bucket_and_reset() {
        let mut m = PacingMetrics::default();
        m.observe_frame(12, false, 0.0);
        assert_eq!(m.histogram[HISTOGRAM_BUCKETS - 1], 1);
        assert_eq!(m.max_ticks_frame, 12);
        m.reset();
        assert_eq!(m.frames, 0);
        assert_eq!(m.max_ticks_frame, 0);
        assert!(m.histogram.iter().all(|&v| v == 0));
        assert_eq!(m.ticks_per_frame_avg(), 0.0);
        assert_eq!(m.blocked_ratio(), 0.0);
    }
}
