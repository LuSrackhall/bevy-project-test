## Tasks

### Step 1: 定义新类型（FactionId / TeamId / SlotId / Controller / PlayerSlots）

**File:** `crates/simulation/src/types.rs`

- 添加 `FactionId(pub u8)`、`TeamId(pub u8)`、`SlotId(pub u8)` struct 定义
- 添加 `Controller` 枚举及其所有变体
- 添加 `PlayerSlot` 和 `PlayerSlots` struct
- 添加 `AiProfile` 和 `AgentId` 占位类型
- 确保所有新类型实现必需的 trait（Clone/Copy/Debug/PartialEq/Eq/Hash 等）
- 实现 `Controller::is_active()` 辅助方法
- 实现 `PlayerSlots::default()` 生产单人模式默认值

### Step 2: FactionComponent 类型迁移

**Files:** `crates/simulation/src/soldier/mod.rs`, `crates/simulation/src/map/mod.rs`, `crates/simulation/src/combat/`, 等所有使用 `FactionComponent` / `Faction` 的文件

- 移除 `Faction` 枚举
- `FactionComponent(pub Faction)` → `FactionComponent(pub FactionId)`
- `Faction::Player` → `FactionId(0)`
- `Faction::Enemy` → `FactionId(1)`
- `Faction::Neutral` → `FactionId(2)`
- 编译并修复所有报错

### Step 3: collect_command_players 改造

**File:** `crates/simulation/src/lib.rs`

- `collect_command_players` 改为接收 `&PlayerSlots`
- `run_tick` 调用处传入 Simulation world 中的 `PlayerSlots` Resource
- NoOp 注入逻辑改为基于 `PlayerSlots`

### Step 4: AI 基于 Slot 改造

**File:** `crates/simulation/src/ai/mod.rs`

- `ai_decide` 函数签名改为接收 `&PlayerSlots`
- AI 迭代所有 AI Controller 的 slot，不再硬编码 `Faction::Enemy`
- `RunConfig.enable_ai` 逻辑适配

### Step 5: bevy_adapter 层适配

**Files:** `crates/bevy_adapter/src/driver.rs`, `crates/bevy_adapter/src/session/bootstrap.rs`

- `simulation_driver_system` 传入 `PlayerSlots`
- SessionBootstrap 初始化时插入 `PlayerSlots`

### Step 6: render_view 层适配

**Files:** `crates/render_view/src/selection.rs`, `crates/render_view/src/camera.rs`, `crates/render_view/src/ui/hud.rs`, `crates/render_view/src/lib.rs`

- 选择/指令系统改用 `PlayerSlots`
- 摄像机居中逻辑依据本地玩家所在 slot
- 胜利判定改用 `FactionId` 比较

### Step 7: 测试与验证

- 运行所有现有测试（simulation 114 + bevy_adapter 22 + relay E2E）
- 验证单人模式启动正常
- 验证 `collect_command_players` 基于 slot 输出正确
