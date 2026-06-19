## Context

项目是一款高并发 RTS 游戏（"城池争霸"），使用 Bevy 0.18 开发，严格分层架构：`simulation ← bevy_adapter ← presentation ← render_view`。

仿真层架构从第一天就为 Lockstep 和 Replay 设计：Fixed(i64) 定点数、GameCommand/CommandBuffer 驱动、DeterministicRng(SmallRng)、纯函数 `run_tick(world, tick) -> SimulationEvents`。但实现层面存在 12-15 处 f32 泄漏直接参与仿真逻辑，破坏了确定性保证。

当前阶段（2 周开发，42 个已完成变更），用户希望引入 Replay 系统作为验证确定性和通向 P2P 联机的第一步。

## Goals / Non-Goals

**Goals:**
- 消除 simulation 层所有 f32 仿真运算，实现单平台位精确确定性
- 通过黄金测试（固定 seed + 指令 → 断言世界状态一致）验证确定性
- 实现完整 Replay 系统：录制、回放、暂停、快进、进度条 seek
- 为未来 P2P Lockstep 铺路（共享 serde 层、命令注入接口）
- 默认自动录制，用户可在菜单设置中关闭

**Non-Goals:**
- 不做 P2P 网络传输（Replay 验证通过后再做）
- 不做世界快照/周期快照（方案 B 留到未来）
- 不做倒退播放（需要快照作为前置条件）
- 不做 Replay 编辑或裁剪
- 不做跨平台确定性（Phase 0b 推迟到跨平台需求明确时）
- 不修复 render_view 直接写入仿真组件的架构债务（单独追踪）
- 不引入 ReplaySource/TickDriver trait（GameMode 枚举足够）

## Decisions

### D1: 确定性修复策略

**决策**：概率值使用 `u32` 万分比（0-10000），比例计算使用整数乘除，不使用 `Fixed` 存储概率。

**理由**：Fixed 的 8-bit 精度（1/256 ≈ 0.004）对概率比较不足。万分比提供 0.01% 精度，覆盖所有配置需求。

**范围**：
- 概率/随机判定（~10 处）：`gen_probability()` 改为返回 `u32`（0-10000），所有配置概率改为万分比整数
- 比例计算（~8 处）：lifesteal_rate、dodge_chance、capture_hp_ratio 等改为万分比整数
- powi 指数运算（1 处）：改为逐次定点数乘法
- HashMap 迭代顺序（1 处）：nearest-enemy 扫描改为 BTreeMap 或加入 tie-break
- SmallRng：锁定 `rand` 版本至 Cargo.lock

### D2: 黄金测试设计

**决策**：固定 seed + 固定指令序列 → 1000 tick 后断言世界状态完全一致（PartialEq 比较所有组件）。

**理由**：在 CI 中自动运行，成本约 30 行代码，可即时捕获确定性回归。覆盖 simulation crate 独立测试，符合宪法要求。

### D3: ReplayFile 归属 simulation 层

**决策**：`ReplayFile` 定义在 `simulation/src/replay.rs`，是纯数据结构。

**结构**：
```rust
#[derive(Serialize, Deserialize)]
pub struct ReplayFile {
    pub format_version: u32,        // 文件格式版本，从 1 开始
    pub seed: u64,                  // RNG 种子
    pub map_size: MapSize,          // 地图大小预设
    pub total_ticks: u32,           // 总 tick 数（进度条用）
    pub commands_per_tick: BTreeMap<u32, Vec<GameCommand>>,  // 外部命令序列
}
```

**理由**：GameCommand、MapSize 等类型全在 simulation 中，serde 已是 simulation 的依赖。纯数据不违反宪法。

### D4: 播放控制归属 bevy_adapter 层

**决策**：独立 `ReplayController` 资源 + `replay_tick_driver_system`，与 `tick_driver_system` 互斥。

**结构**：
```rust
#[derive(Resource)]
pub enum GameMode {
    Live,
    Replay { controller: ReplayController },
}

pub struct ReplayController {
    pub replay_data: ReplayFile,
    pub target_tick: Option<u32>,   // seek 目标，None = 播放到结尾
    pub speed_multiplier: f32,      // 1.0, 2.0, 4.0
    pub is_paused: bool,
    pub is_seeking: bool,
}
```

**理由**：TickClock 是实时对局时钟，逻辑清晰。回放的 tick 推进策略完全不同（每帧 N tick、seek 到目标 tick），硬塞进 TickClock 会让状态机复杂度暴涨。

### D5: 录制机制归属 bevy_adapter 层

**决策**：在 `tick_driver_system` 中拦截已提取的命令副本，不修改 `run_tick()` 接口。

**要点**：
- 仅录制外部玩家命令，不录制 AI 命令（AI 在 run_tick 内部产生，是确定性的）
- GameOver 时将录制缓冲区序列化到磁盘
- SimulationSeed 资源持久化（当前 seed 被丢弃，Replay 需要）

### D6: render_view 进度条依赖

**决策**：render_view 通过 bevy_adapter 的轻量资源读取 `is_replay: bool` 和 `total_ticks: u32`，不 import `ReplayFile`。

### D7: 文件格式

**决策**：开发阶段使用 RON（可读可调试），未来可切换 bincode（紧凑高效）。文件头包含 `format_version` 字段确保向后兼容。

### D8: 不引入 trait 抽象

**决策**：使用 `GameMode` 枚举控制命令来源，不引入 `CommandSource` trait。

**理由**：当前只有两种模式（Live/Replay），枚举足够。trait 留到三种以上时再提取（YAGNI）。

## Risks / Trade-offs

| 风险 | 严重度 | 缓解措施 |
|------|--------|----------|
| f32 修复可能影响现有游戏平衡 | 中 | 万分比保留 0.01% 精度，足够覆盖配置需求。修改后需运行现有测试验证 |
| seek 性能：30 分钟对局 seek 需 18-54 秒 | 中 | Phase 3 引入 silent tick + 周期快照优化 |
| Replay 文件在游戏代码更新后静默失效 | 中 | 文件头记录 format_version + game_version，加载时警告版本不匹配 |
| SmallRng 跨 rand 版本不确定 | 低 | 锁定 Cargo.lock 中 rand 版本 |
| render_view 直接写入仿真组件破坏命令管道唯一入口 | 中 | 单独追踪为架构债务，不在 Replay 项目中修复 |

## 四阶段实施路线

### Phase 0: 修复仿真确定性（2-3 天）
- combat/soldier/city config 的 f32 概率字段改为 u32 万分比
- gen_probability() 返回 u32（0-10000）
- ai/mod.rs 的 f32 比较改为整数比较
- HashMap nearest-enemy 扫描改为 BTreeMap
- 锁定 rand 版本至 Cargo.lock

### Phase 1: 黄金确定性测试（1 天）
- 实现世界状态哈希函数
- 编写 3-5 个黄金测试用例（空地图、1v1 战斗、多城市混战）
- CI 持续运行

### Phase 2: Replay 基础功能（3-4 天）
- simulation 层：为 GameCommand/Action 及依赖类型加 serde，定义 ReplayFile
- bevy_adapter 层：GameMode 枚举、ReplayRecorder、ReplayController、replay_tick_driver_system
- render_view 层：Replay 播放器 UI（播放/暂停、快进、进度条）、"Load Replay" 按钮、设置开关

### Phase 3: Replay 增强（后续迭代）
- silent tick（跳过 SimulationEvents 构建）
- 周期快照 + 快速 seek
- bincode 生产格式
- 版本兼容层
