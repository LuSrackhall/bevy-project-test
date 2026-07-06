## Tasks

### Step 1: 定义新类型（FactionId / TeamId / SlotId / Controller / PlayerSlots）

**File:** `crates/simulation/src/types.rs`

- [x] 添加 `FactionId(pub u8)`、`TeamId(pub u8)`、`SlotId(pub u8)` struct 定义
- [x] 添加 `Controller` 枚举及其所有变体
- [x] 添加 `PlayerSlot` 和 `PlayerSlots` struct
- [x] 添加 `AiProfile` 和 `AgentId` 占位类型
- [x] 确保所有新类型实现必需的 trait（Clone/Copy/Debug/PartialEq/Eq/Hash 等）
- [x] 实现 `Controller::is_active()` 辅助方法
- [x] 实现 `PlayerSlots::single_player()` 生产单人模式默认值

### Step 2: FactionComponent 类型迁移

**Files:** `crates/simulation/src/soldier/mod.rs`, `crates/simulation/src/map/mod.rs`, `crates/simulation/src/combat/`, 等 35+ 文件

- [x] 移除 `Faction` 枚举
- [x] `FactionComponent(pub Faction)` → `FactionComponent(pub FactionId)`
- [x] `Faction::Player` → `FactionId(0)`（全局 35+ 文件）
- [x] `Faction::Enemy` → `FactionId(1)`
- [x] `Faction::Neutral` → `FactionId(2)`
- [x] 编译并修复所有报错
- [x] 添加所有 FactionId match 的通配臂

### Step 3: collect_command_players 改造

**File:** `crates/simulation/src/lib.rs`

- [x] `collect_command_players` 从 PlayerSlots 读取活跃 faction
- [x] NoOp 注入基于 PlayerSlots
- [x] 有 Legacy fallback（未设置 PlayerSlots 的场景兼容）

### Step 4: AI 基于 Slot 改造

**File:** `crates/simulation/src/ai/mod.rs`

- [x] `ai_decide`读取 PlayerSlots，遍历所有 AI Controller slot
- [x] AI 不再硬编码 `Faction::Enemy`，使用 `ai_faction` 参数
- [x] `RunConfig.enable_ai` 逻辑兼容
- [x] AI 命令 `player_id` 使用 `ai_faction.0`

### Step 5: bevy_adapter 层适配

**Files:** `crates/bevy_adapter/src/driver.rs`, `crates/bevy_adapter/src/session/bootstrap.rs`

- [x] PlayerSlots 自动通过 simulation Resource 可用
- [/] SessionBootstrap 网络模式下显式初始化 PlayerSlots（当前使用默认 single_player）

### Step 6: render_view 层适配

**Files:** `crates/render_view/src/selection.rs`, `crates/render_view/src/camera.rs`, `crates/render_view/src/ui/hud.rs`, `crates/render_view/src/lib.rs`

- [x] 选择/指令系统使用 `LocalPlayerId` → `FactionId(lid)` 兼容路径
- [x] 摄像机居中依据本地玩家 faction
- [x] 胜利判定改用 FactionId 比较
- [ ] HUD 按钮 `player_id: 0` 硬编码——标记已知问题，下个迭代修复

### Step 7: 测试与验证

- [x] 运行所有现有测试（simulation 114 + bevy_adapter 22 + relay E2E）
- [x] 编译通过（含 --all-targets）
- [x] network_e2e 编译修复
- [ ] 添加多 AI slot 测试场景（建议后续补充）
