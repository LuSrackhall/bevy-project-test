## 1. 下拉框补充民兵

- [x] 1.1 `setup_hud` 中下拉选项列表补充"民兵"项（`SeekScope::ByType(Militia)`）

## 2. 范围输入框实时生效

- [x] 2.1 `SeekPanelState` 移除 `editing` 和 `input_buffer`，新增 `input_active: bool`
- [x] 2.2 重写 `seek_panel_input_system`：点击输入框设置 `input_active`，数字键直接修改 `range_value`，Backspace 删除末位
- [x] 2.3 移除 `handle_pause_input` 中对 `editing` 的检查，改为检查 `input_active`

## 3. 模式标签

- [x] 3.1 `HudTexts` 新增 `mode_label: Option<Entity>`
- [x] 3.2 `setup_hud` 中在 scope 选择框左侧添加模式标签 Text 节点
- [x] 3.3 `seek_panel_mode_system` 中更新模式标签文字（"索敌"/"选中"）

## 4. 下拉选项实时数量

- [x] 4.1 `HudTexts` 新增 5 个 `seek_option_count` 字段（每个选项的数量文本实体）
- [x] 4.2 `setup_hud` 中每个选项旁边添加数量 Text 子实体
- [x] 4.3 新增 `seek_panel_count_system`：在 dropdown 打开时查询 simulation 世界更新数量文本

## 5. 验证

- [x] 5.1 `cargo build` 编译通过
- [x] 5.2 `cargo test -p simulation` 全部通过
