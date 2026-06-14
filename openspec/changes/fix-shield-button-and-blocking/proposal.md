## Why

1. 盾牌按钮（"盾"）是空占位符，点击无效
2. 格挡状态下的行为逻辑需要更新（朝向锁定、攻击限制、移动限制）

## What Changes

### UI 层（render_view）

**盾牌按钮逻辑：**
- 仅当选择的单位中包含步兵时显示盾牌按钮
- 若选择的步兵中部分举盾、部分未举盾 → 显示"举盾"按钮（可点击）
- 若选择的步兵全部举盾 → 显示"取消举盾"按钮（可点击）
- 点击发送 `SetShield` 命令到 simulation

### Simulation 层

**格挡状态行为更新：**

1. **朝向锁定**：`facing_turn_system` 跳过 Blocking 状态的单位，朝向不再自动调整
2. **攻击限制**：Blocking 状态下只攻击正面 120° 扇形内的敌人（当前已实现）
3. **普通移动不改变朝向**：MoveTo 命令不改变 Blocking 单位的朝向
4. **强制移动解锁朝向**：ForceMove 命令自动取消 Blocking 状态，恢复朝向调整
5. **取消格挡**：SetShield(Normal) 取消 Blocking，恢复朝向调整

## Capabilities

### Modified
- `toolbar_button_system`: 实现盾牌按钮逻辑
- `facing_turn_system`: 跳过 Blocking 状态
- `combat_engagement_system`: Blocking 状态下只攻击正面范围内敌人
- `consume_commands_system`: ForceMove 自动取消 Blocking

## Impact

- `render_view/src/ui/hud.rs` — 盾牌按钮显示/隐藏、点击逻辑
- `simulation/src/facing.rs` — facing_turn_system 跳过 Blocking
- `simulation/src/combat/mod.rs` — engagement 攻击范围限制
- `simulation/src/soldier/mod.rs` — ForceMove 取消 Blocking
