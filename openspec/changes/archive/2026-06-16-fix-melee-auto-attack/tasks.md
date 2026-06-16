## Tasks

### 1. 新增定点数 cos 近似函数
- [x] 在 `facing.rs` 中新增 `cos_approx(angle: Fixed) -> Fixed` 函数
- [x] 使用 Bhaskara I 公式：cos(x°) ≈ (32400 - 4x²) / (32400 + 4x²)
- [x] 添加单元测试验证精度（0°, 90°, 180°）

### 2. 重写近战系统目标选择
- [x] 移除对 `Movement.target` 的依赖
- [x] 改为扫描所有敌人士兵位置，筛选攻击范围内的敌人
- [x] 选择最近的敌人作为攻击目标
- [x] 保持 ForceMove 抑制逻辑（非骑兵在 ForceMove 时跳过）

### 3. 添加朝向攻速影响
- [x] 在近战攻击时，计算朝向与攻击方向的偏差角
- [x] 攻速因子 = 1 + 0.3 × cos(偏差角)
- [x] 应用攻速因子到攻击冷却时间（melee_attack_system 和 attack_windup_system）
- [x] 骑兵不受影响（无前摇）

### 4. 验证
- [x] `cargo test -p simulation` — 74 测试全部通过
- [x] `cargo build` — 全项目编译通过
- [ ] 手动测试：站立不动时自动攻击范围内敌人
- [ ] 手动测试：移动中自动攻击范围内敌人
- [ ] 手动测试：ForceMove 不攻击（骑兵除外）
- [ ] 手动测试：正面对敌攻速快，背对攻速慢
