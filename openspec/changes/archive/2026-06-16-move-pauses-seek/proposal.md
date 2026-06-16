## Why

当前玩家下达移动命令后，单位仍会因索敌系统中途转向追击敌人，违背玩家意图。需要让移动命令临时暂停索敌，到达后自动恢复。同时将索敌默认范围改为 0，确保索敌默认关闭，符合"无命令不行动"原则。

## What Changes

- **修改** `consume_commands_system` 中 `MoveTo` 和 `ForceMove` 的处理：设置 `force_move = true`（临时暂停索敌）
- **修改** `soldier_movement_system`：单位到达目标点后，如果 `SeekStance.active`，恢复 `force_move = false`
- **修改** `seek_panel_mode_system` 中的默认范围值：全局模式 10→0，选择模式 30→0
- **保留** 移动途中的自动攻击行为（`melee_attack_system` 不受影响）

## Capabilities

### New Capabilities

- `move-pauses-seek`: 移动命令临时暂停索敌，到达后自动恢复

### Modified Capabilities

（无）

## Impact

- **simulation/src/soldier/mod.rs**: `consume_commands_system`、`soldier_movement_system`
- **render_view/src/ui/hud.rs**: 默认范围值
