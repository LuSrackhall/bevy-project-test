## 1. 修复框选窗口逃逸

- [x] 1.1 修改 `drag_select_system`：分离 `cursor_position()` 为 `None` 的处理逻辑
- [x] 1.2 修改 `drag_select_system`：将 `just_released` 改为 `!pressed`
- [x] 1.3 编译验证
- [ ] 1.4 运行验证：框选过程中鼠标移出窗口，框选矩形保持
- [ ] 1.5 运行验证：鼠标在窗口外松开，框选完成
- [ ] 1.6 运行验证：鼠标在窗口内松开，行为不变
