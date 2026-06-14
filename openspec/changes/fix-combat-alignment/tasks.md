## Tasks

### 1. 恢复近战前摇
- [ ] `melee_attack_system`：非骑兵创建 `AttackWindup` 条目而非直接攻击
- [ ] 骑兵仍立即攻击（`cavalry_no_windup = true`）
- [ ] 前摇期间单位不移动（`soldier_movement_system` 中检查 `AttackWindup.remaining_ticks > 0`）
- [ ] `attack_windup_system` 处理前摇完成后的伤害（已有逻辑）

### 2. 修正盾牌手动格挡移速
- [ ] `soldier_movement_system`：手动格挡时移速直接设为 `speed_penalty`（15），而非 `speed - speed_penalty`

### 3. 非正面伤害跳过被动格挡
- [ ] `try_passive_block`：当 `ShieldState::Blocking` 且攻击来自非正面时，跳过被动格挡，直接扣士兵 HP

### 4. 文档同步
- [ ] 更新 `rts-game-design.md`：近战自动扫描、朝向攻速因子、MoveTo/ForceMove、箭矢散布

### 5. 验证
- [ ] `cargo test -p simulation` — 所有测试通过
- [ ] 更新受影响的测试用例
