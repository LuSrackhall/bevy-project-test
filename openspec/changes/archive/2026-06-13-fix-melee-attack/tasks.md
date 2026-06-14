## Tasks

### 1. 扩展射程内保持目标逻辑 ✅
- [x] 将弓兵专属的"在射程内 continue"逻辑扩展到所有 Fighting 状态单位
- [x] 非弓兵使用 `attack_range`（固定值），弓兵使用 `compute_attack_range(level)`（等级缩放）

### 2. Fighting 状态禁用到达处理 ✅
- [x] 在到达判定前检查 `sst.0 == SoldierState::Fighting && mov.target.is_some()`
- [x] 如果是 Fighting 且有目标，跳过到达（不清除目标、不改变状态）

### 3. 修复骑兵索敌目标设置 ✅
- [x] 移除 `combat_engagement_system` 中的 `if !is_cav` 守卫
- [x] 骑兵现在与其它兵种一样正确接收 `Movement.target`

### 4. 验证 ✅
- [x] `cargo test -p simulation` — 68 测试全部通过
- [x] `cargo build` — 全项目编译通过
