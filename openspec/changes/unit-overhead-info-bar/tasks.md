## 1. 模块搭建

- [x] 1.1 创建 `render_view/src/unit_info_bar.rs`，定义 `UnitInfoBar` 组件和 `UnitInfoBarSettings` 资源（含 `InfoBarMode` 枚举和 Default）
- [x] 1.2 在 `render_view/src/lib.rs` 中注册 `unit_info_bar` 模块，添加 `init_resource::<UnitInfoBarSettings>()` 和 `update` 系统，确保排在 `draw_debug_shapes` 之后

## 2. 信息条渲染系统

- [x] 2.1 实现 `unit_info_bar_system`：查询 `(Entity, &GlobalTransform, &Health, &Level)` + `Option<&UnitInfoBar>`，为无标记实体创建信息条子实体并挂载 `UnitInfoBar`
- [x] 2.2 实现血量条创建：使用 `ShapeBundle` 创建红色底矩形（40×6px）和绿色填充矩形（宽度按 `Health.current/Health.max` 比例），作为子实体挂载
- [x] 2.3 实现经验条创建：使用 `ShapeBundle` 创建蓝色底矩形（40×4px）和紫色填充矩形（宽度按 `Level.exp` 比例），作为子实体挂载
- [x] 2.4 实现文字创建：使用 `Text2D` 创建等级文字（10px 白色，显示 "Lv.X"）和数值叠加文字（8px 白色，显示 "cur/max"），作为子实体挂载
- [x] 2.5 实现条宽/文字更新：每帧根据最新 `Health` 和 `Level` 值更新填充矩形宽度和数值文字内容

## 3. 布局与定位

- [x] 3.1 实现子实体定位：血条在单位头顶上方 12px，经验条在血条下方 2px，等级文字在血条左上方对齐
- [x] 3.2 使用 `Transform` 的局部坐标设置各子实体相对于父实体的偏移，确保随父实体移动

## 4. 显示模式

- [x] 4.1 实现显示模式判断逻辑：`Always` 始终可见；`Selected` 仅 `SelectionState` 中选中单位可见；`Smart` 选中单位可见 + 未选中单位 HP<max 或 EXP>0 时可见
- [x] 4.2 实现可见性控制：根据当前模式设置信息条子实体的 `Visibility` 为 `Visible` 或 `Hidden`
- [x] 4.3 实现 `Ctrl+H` 快捷键：在 `unit_info_bar_system` 或独立输入系统中监听 `Ctrl+H`，循环切换 `InfoBarMode`

## 5. 底部面板精简

- [x] 5.1 修改 `render_view/src/ui/hud.rs` 的 `update_bottom_panel`：移除士兵面板中对 `Health` 和 `Level.exp` 的文本更新逻辑
- [x] 5.2 修改城市面板更新逻辑：移除对 `CityComponent` 中等级、血量、经验的文本更新，保留人口和训练类型

## 6. 验证与清理

- [x] 6.1 `cargo check` 确认编译通过
- [x] 6.2 `cargo test` 确认已有测试全部通过
- [ ] 6.3 运行游戏手动验证：单位信息条正确显示、模式切换正常、底部面板精简正确
