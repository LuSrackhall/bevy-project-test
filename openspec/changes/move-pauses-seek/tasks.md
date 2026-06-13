## 1. Simulation 层 — 移动暂停索敌

- [x] 1.1 `consume_commands_system` 中 `MoveTo` 分支设置 `force_move = true`
- [x] 1.2 `soldier_movement_system` 到达逻辑中：已自动重置 `force_move = false`（索敌恢复）

## 2. Render View 层 — 默认范围

- [x] 2.1 `SeekPanelState` 默认 `range_value` 从 10 改为 0
- [x] 2.2 `seek_panel_mode_system` 模式切换默认值改为 0

## 3. 验证

- [x] 3.1 `cargo test -p simulation` 全部通过
- [x] 3.2 `cargo build` 编译通过
