## 1. 选中逻辑统一

- [x] 1.1 `SelectionState` 新增 `selected_city: Option<UnitId>` 字段
- [x] 1.2 `selection_click_system`: 先查城池，再查兵种，城池命中清空兵种选中
- [x] 1.3 `drag_select_system`: 查询加 `SoldierMarker` 过滤

## 2. 面板统一

- [x] 2.1 `hud.rs`: 移除中间40%区城池面板 + 右边30%区小地图；简化为左30%信息面板 + 右70%命令卡片+图鉴
- [x] 2.2 左面板根据 `selected_city`/`selected_unit_ids` 切换城池/兵种/占位内容

## 3. 资源清理

- [x] 3.1 移除 `SelectedCity` 资源
- [x] 3.2 移除 `city_click_system` 系统
- [x] 3.3 `ui/mod.rs` 清理相关注册

## 4. 验证

- [x] 4.1 `cargo check` 编译通过
- [x] 4.2 `cargo test` 通过
