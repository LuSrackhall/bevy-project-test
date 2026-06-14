## Tasks

### 1. 盾牌按钮 UI 逻辑
- [x] 盾牌按钮仅在选择的单位中有步兵时显示
- [x] 部分举盾 → 显示"举盾"按钮
- [x] 全部举盾 → 显示"取消举盾"按钮
- [x] 点击发送 `SetShield` 命令

### 2. 朝向锁定
- [x] `facing_turn_system` 跳过 Blocking 状态单位
- [x] Blocking 状态下 MoveTo 不改变朝向

### 3. 攻击范围限制
- [x] Blocking 状态下只攻击正面 120° 扇形内敌人

### 4. ForceMove 解锁朝向
- [x] ForceMove 命令自动取消 Blocking 状态

### 5. 验证
- [x] `cargo test -p simulation` — 83 测试全部通过
- [x] `cargo build` — 全项目编译通过
