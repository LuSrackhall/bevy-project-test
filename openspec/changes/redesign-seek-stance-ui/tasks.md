## 1. 资源与状态定义

- [x] 1.1 在 `render_view/src/ui/hud.rs` 中新增 `SeekPanelState` 资源（`scope: SeekScope`, `dropdown_open: bool`, `editing: bool`, `input_buffer: String`, `range_value: u32`），实现 `Default`（scope=All, range=10）
- [x] 1.2 在 `render_view/src/ui/hud.rs` 中新增 `ToastMessage` 资源（`text: String`, `remaining_ticks: u32`），实现 `Default`
- [x] 1.3 在 `render_view/src/ui/hud.rs` 中新增面板 UI 组件标记：`SeekPanelRoot`、`SeekScopeDropdown`、`SeekScopeOption(SeekScope)`、`SeekRangeInput`、`SeekIssueBtn`、`ToastText`

## 2. 移除旧 UI，构建新面板

- [x] 2.1 从 `setup_hud` 的 toolbar 区域移除旧的 5 个 `SeekBtn` 按钮和 `SeekStatusText` 文字
- [x] 2.2 在 `setup_hud` toolbar 右侧（分隔线后）构建新索敌面板：scope 下拉触发器 + 范围输入框 + 下发按钮
- [x] 2.3 在 `setup_hud` 顶部状态栏右侧新增 `ToastText` 文本节点
- [x] 2.4 注册 `SeekPanelState` 和 `ToastMessage` 为 `init_resource`（在 `ui/mod.rs` 中）

## 3. 索敌面板交互系统

- [x] 3.1 新增 `seek_panel_mode_system`：根据 `SelectionState.selected_unit_ids` 是否为空，在面板中切换全局/选择模式（更新范围默认值：全局 10，选择 30，仅在模式切换瞬间重置）
- [x] 3.2 新增 `seek_panel_dropdown_system`：处理下拉框点击展开/收起、选项点击选中、点击外部收起
- [x] 3.3 新增 `seek_panel_input_system`：处理范围输入框点击进入编辑态、数字键输入、Backspace 删除、Enter 确认、Escape 取消
- [x] 3.4 新增 `seek_panel_issue_system`：处理下发按钮点击，生成 `Action::SetSeekStance` 命令推入 CommandBuffer，触发 Toast 消息

## 4. Toast 消息系统

- [x] 4.1 新增 `toast_tick_system`：每帧递减 `remaining_ticks`，归零时清空文字
- [x] 4.2 新增 `toast_display_system`：读取 `ToastMessage` 更新 `ToastText` 节点文字，`remaining_ticks > 0` 时显示，否则显示空
- [x] 4.3 在 `seek_panel_issue_system` 中设置下发确认 Toast 消息（根据模式和 scope 格式化消息文字）
- [x] 4.4 新增选中变化时的摘要 Toast：监听 `SelectionState` 变化，非空时生成"选中 N 个兵种"或"选中 N 个单位: 兵种1数量1 兵种2数量2"消息

## 5. 编辑模式冲突处理

- [x] 5.1 在 `seek_stance_shortcut_system` 中检查 `SeekPanelState.editing == false` 再响应 S 键
- [x] 5.2 在 `handle_pause_input` 中检查编辑状态，Escape 优先取消编辑而非暂停/取消选中

## 6. 注册系统与验证

- [x] 6.1 在 `ui/mod.rs` 中注册新增的 8 个系统（mode、dropdown、input、issue、toast_tick、toast_display、selection_summary_toast），`run_if(in_state(GameState::Playing))`
- [x] 6.2 `cargo test -p simulation` 全部通过（39/39）
- [x] 6.3 `cargo build` 全项目编译通过
