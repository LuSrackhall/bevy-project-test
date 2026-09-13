//! 链路条件注入（延迟 / 抖动 / 丢包）——用于复现"恶劣网络下的联机卡顿"。
//!
//! 为什么需要：loopback 上 RTT≈0、几乎不丢包，本地稳态**测不出**真实 WiFi/公网的症状。
//! 本模块提供**确定性可复现**的注入——同一 seed + 同一发送次数 ⇒ 同一判定序列，
//! 这样"卡顿"才能变成可重复的实验与门禁，而不是"感觉"。
//!
//! 只在意显式配置时生效：`None` 表示真实链路，生产路径零开销。

/// 注入参数。全为 0 时等价于不注入。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetemConfig {
    /// 基础单向延迟（毫秒）。
    pub latency_ms: u32,
    /// 抖动幅度（毫秒）：实际延迟落在 `latency ± jitter` 内。
    pub jitter_ms: u32,
    /// 丢包率（百分比，0..=100）。丢包交由可靠层重传处理，因此会同时放大延迟。
    pub loss_pct: u8,
    /// 伪随机种子：同 seed + 同发送序 ⇒ 同判定（可复现实验）。
    pub seed: u64,
}

impl NetemConfig {
    pub fn new(latency_ms: u32, jitter_ms: u32, loss_pct: u8, seed: u64) -> Self {
        Self {
            latency_ms,
            jitter_ms,
            loss_pct: loss_pct.min(100),
            seed,
        }
    }

    /// 是否无需注入（生产路径可据此跳过一切开销）。
    pub fn is_noop(&self) -> bool {
        self.latency_ms == 0 && self.jitter_ms == 0 && self.loss_pct == 0
    }
}

/// 注入器状态：自持 xorshift64* 伪随机序列，避免依赖全局 rand 状态。
#[derive(Clone, Debug)]
pub struct NetemState {
    cfg: NetemConfig,
    rng: u64,
    sent: u64,
}

impl NetemState {
    pub fn new(cfg: NetemConfig) -> Self {
        // 0 会让 xorshift 退化，强制置为非零。
        Self {
            cfg,
            rng: cfg.seed.wrapping_mul(2_862_933_555_777_941_757) | 1,
            sent: 0,
        }
    }

    pub fn config(&self) -> NetemConfig {
        self.cfg
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 32) as u32
    }

    /// 本次发送是否丢弃。计数用于复现与统计。
    pub fn should_drop(&mut self) -> bool {
        self.sent += 1;
        if self.cfg.loss_pct == 0 {
            return false;
        }
        (self.next_u32() % 100) < u32::from(self.cfg.loss_pct)
    }

    /// 本次发送的延迟（毫秒），落在 `latency ± jitter` 内（下界饱和到 0）。
    pub fn delay_ms(&mut self) -> u32 {
        if self.cfg.jitter_ms == 0 {
            return self.cfg.latency_ms;
        }
        let span = self.cfg.jitter_ms * 2 + 1;
        let offset = self.next_u32() % span;
        self.cfg
            .latency_ms
            .saturating_sub(self.cfg.jitter_ms)
            .saturating_add(offset)
    }

    /// 已判定的发送次数。
    pub fn sent(&self) -> u64 {
        self.sent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 同 seed ⇒ 完全相同的判定序列（实验可复现）。
    #[test]
    fn test_same_seed_is_reproducible() {
        let cfg = NetemConfig::new(40, 15, 5, 42);
        let mut a = NetemState::new(cfg);
        let mut b = NetemState::new(cfg);
        let seq_a: Vec<(bool, u32)> = (0..500).map(|_| (a.should_drop(), a.delay_ms())).collect();
        let seq_b: Vec<(bool, u32)> = (0..500).map(|_| (b.should_drop(), b.delay_ms())).collect();
        assert_eq!(seq_a, seq_b, "同 seed 必须产生相同序列");
    }

    /// 丢包率与延迟区间必须符合配置（统计意义上）。
    #[test]
    fn test_loss_rate_and_delay_bounds() {
        let cfg = NetemConfig::new(50, 20, 10, 7);
        let mut s = NetemState::new(cfg);
        let mut dropped = 0u32;
        let n = 10_000u32;
        for _ in 0..n {
            if s.should_drop() {
                dropped += 1;
            }
            let d = s.delay_ms();
            assert!(
                (30..=70).contains(&d),
                "延迟 {d}ms 超出 50±20 区间"
            );
        }
        let pct = dropped as f64 / f64::from(n) * 100.0;
        assert!(
            (8.0..12.0).contains(&pct),
            "丢包率 {pct:.2}% 偏离配置 10%"
        );
        assert_eq!(s.sent(), u64::from(n));
    }

    /// 全 0 配置必须是无操作（生产路径不付出任何代价）。
    #[test]
    fn test_noop_config() {
        let cfg = NetemConfig::new(0, 0, 0, 1);
        assert!(cfg.is_noop());
        let mut s = NetemState::new(cfg);
        for _ in 0..1000 {
            assert!(!s.should_drop());
            assert_eq!(s.delay_ms(), 0);
        }
    }
}
