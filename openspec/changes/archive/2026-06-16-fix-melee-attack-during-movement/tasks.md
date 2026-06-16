## Tasks

### 1. 修复 MoveTo 的 force_move 参数
- [x] 在 `consume_commands_system` 中，将 `MoveTo` 的 `apply_movement` 调用从 `true` 改为 `false`

### 2. 实现移动中攻击逻辑
- [x] 在 `melee_attack_system` 中，移除对 `SoldierState::Fighting` 的依赖
- [x] 改为检查 `Movement.target` 是否为 `Some`
- [x] 如果目标在攻击范围内且冷却归零 → 执行攻击
- [x] 攻击后不改变移动相关组件

### 3. ForceMove 抑制
- [x] 在移动中攻击逻辑中，检查 `force_move` 标志
- [x] `force_move == true` 且非骑兵 → 跳过攻击
- [x] `force_move == true` 且骑兵 → 允许攻击
- [x] 在 `attack_windup_system` 中也添加 ForceMove 抑制（取消已开始的蓄力）

### 4. 验证
- [x] `cargo test -p simulation` — 68 测试全部通过
- [x] `cargo build` — 全项目编译通过
- [ ] 手动测试：MoveTo 移动中自动攻击
- [ ] 手动测试：ForceMove 不攻击（骑兵除外）
