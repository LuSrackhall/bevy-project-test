## 1. 新增 Classic 模式

- [x] 1.1 在 `InfoBarMode` 枚举中新增 `Classic` 变体
- [x] 1.2 更新 `InfoBarMode::next()` 循环顺序：Always → Selected → Smart → Classic → Always
- [x] 1.3 将 `UnitInfoBarSettings::default()` 的默认模式从 `Smart` 改为 `Classic`

## 2. 实现悬停检测

- [x] 2.1 在 `unit_info_bar_system` 签名中添加 `Query<&Window>` 和 `Query<(&Camera, &GlobalTransform), With<MainCamera>>` 参数
- [x] 2.2 在系统开始处将光标屏幕坐标转换为世界坐标（复用 `selection.rs` 的 `screen_to_world` 模式）
- [x] 2.3 遍历所有单位，比较光标世界坐标与单位位置的距离，构建 `hovered_ids: HashSet<UnitId>`
- [x] 2.4 悬停阈值：士兵使用 144.0（12px 半径平方），城池使用 `CityRadius` 平方

## 3. 更新 should_show 逻辑

- [x] 3.1 在 `should_show` 计算前，定义 `is_hovered = hovered_ids.contains(&info.unit_id)`
- [x] 3.2 修改 `should_show` 匹配逻辑：
  - `Always` → `true`
  - `Selected` → `is_selected || is_hovered`
  - `Smart` → `is_selected || info.hp_cur < info.hp_max || info.exp > 0`
  - `Classic` → `is_selected || is_hovered`

## 4. 验证

- [x] 4.1 `cargo check` 确认编译通过
- [ ] 4.2 运行游戏手动验证：Classic 模式下选中/悬停显示血条，Selected 模式增加悬停支持
