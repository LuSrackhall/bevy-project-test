## 1. 修复 create_bar 中的填充矩形

- [ ] 1.1 修改 `create_bar` 中 HP fill：将 `RectangleOrigin::CustomCenter(...)` 替换为 `RectangleOrigin::Center`，将 `Transform::from_xyz(0.0, 2.0, 0.01)` 改为 `Transform::from_xyz(-HP_BAR_W / 2.0 + hp_w / 2.0, 2.0, 0.01)`
- [ ] 1.2 修改 `create_bar` 中 EXP fill：将 `RectangleOrigin::CustomCenter(...)` 替换为 `RectangleOrigin::Center`，将 `Transform::from_xyz(0.0, -3.0, 0.01)` 改为 `Transform::from_xyz(-EXP_BAR_W / 2.0 + exp_w / 2.0, -3.0, 0.01)`

## 2. 修复 update_bar 中的填充矩形

- [ ] 2.1 修改 `update_bar` 中 HP fill spawn：将 `RectangleOrigin::CustomCenter(...)` 替换为 `RectangleOrigin::Center`，将 `Transform::from_xyz(0.0, 2.0, 0.01)` 改为 `Transform::from_xyz(-HP_BAR_W / 2.0 + hp_w / 2.0, 2.0, 0.01)`
- [ ] 2.2 修改 `update_bar` 中 EXP fill spawn：将 `RectangleOrigin::CustomCenter(...)` 替换为 `RectangleOrigin::Center`，将 `Transform::from_xyz(0.0, -3.0, 0.01)` 改为 `Transform::from_xyz(-EXP_BAR_W / 2.0 + exp_w / 2.0, -3.0, 0.01)`

## 3. 验证

- [ ] 3.1 `cargo check` 确认编译通过
- [ ] 3.2 运行游戏手动验证：血量条和经验条填充按实际占比正确显示
