## 1. 插件注册

- [ ] 1.1 在 `render_view/src/lib.rs` 注册 `InputDispatchPlugin` + `TabNavigationPlugin` + `MenuPlugin` + `PopoverPlugin`
- [ ] 1.2 编译验证：确认 `InputFocus` 资源正确初始化，无 panic
- [ ] 1.3 运行验证：游戏正常启动，现有按钮功能不受影响

## 2. 下拉菜单迁移到 MenuPopup

- [ ] 2.1 重构下拉菜单 spawn 代码：anchor 实体 → MenuButton + MenuPopup → MenuItem
- [ ] 2.2 为每个 MenuItem 添加 `SeekScopeOption` 组件和 `Activate` observer
- [ ] 2.3 配置 `Popover` 定位（向上弹出）
- [ ] 2.4 添加 `OverrideClip` 和 `GlobalZIndex(100)` 确保 popup 在其他 UI 之上
- [ ] 2.5 更新 scope 文本显示（选中后更新触发按钮文字）
- [ ] 2.6 编译验证

## 3. 清理旧实现

- [ ] 3.1 删除 `seek_panel_dropdown_system` 函数
- [ ] 3.2 从 `ui/mod.rs` 删除 `seek_panel_dropdown_system` 注册
- [ ] 3.3 从 `SeekPanelState` 删除 `dropdown_open` 和 `trigger_clicked` 字段
- [ ] 3.4 删除 `SeekDropdownPopup` 组件
- [ ] 3.5 删除下拉选项的 `Pointer<Click>` observer
- [ ] 3.6 编译验证

## 4. 验证

- [ ] 4.1 运行验证：下拉菜单展开/收起正常
- [ ] 4.2 运行验证：选项选择正常，scope 更新
- [ ] 4.3 运行验证：点击外部关闭正常
- [ ] 4.4 运行验证：输入框键盘捕获不受影响
- [ ] 4.5 运行验证：其他 UI 功能不受影响
