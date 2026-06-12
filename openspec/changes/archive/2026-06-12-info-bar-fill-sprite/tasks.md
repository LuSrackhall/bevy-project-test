## 1. 修改 create_bar 中的填充矩形

- [x] 1.1 将 `create_bar` 中 HP fill 从 `ShapeBuilder::fill().build()` 改为 `Sprite { color: HP_FILL, custom_size: Some(Vec2::new(hp_w, HP_BAR_H)), anchor: Anchor::Center }`，Transform x 设为 `-HP_BAR_W / 2.0 + hp_w / 2.0`
- [x] 1.2 将 `create_bar` 中 EXP fill 从 `ShapeBuilder::fill().build()` 改为 `Sprite { color: EXP_FILL, custom_size: Some(Vec2::new(exp_w, EXP_BAR_H)), anchor: Anchor::Center }`，Transform x 设为 `-EXP_BAR_W / 2.0 + exp_w / 2.0`

## 2. 修改 update_bar 中的填充矩形更新逻辑

- [x] 2.1 将 `update_bar` 中 HP fill 更新从 despawn+respawn 改为直接修改 `Sprite.custom_size` 和 `Transform.translation.x`（通过 `Query<(&mut Sprite, &mut Transform), With<HpFill>>`）
- [x] 2.2 将 `update_bar` 中 EXP fill 更新从 despawn+respawn 改为直接修改 `Sprite.custom_size` 和 `Transform.translation.x`（通过 `Query<(&mut Sprite, &mut Transform), With<ExpFill>>`）

## 3. 清理 marker 组件

- [x] 3.1 移除 `HpFill` marker 组件定义（不再需要，改用 `With<BarRoot>` 的子实体查询或保留但不再用于 despawn 逻辑）
- [x] 3.2 移除 `ExpFill` marker 组件定义

## 4. 验证

- [x] 4.1 `cargo check` 确认编译通过
- [ ] 4.2 运行游戏手动验证：血量条和经验条填充按实际占比正确显示
