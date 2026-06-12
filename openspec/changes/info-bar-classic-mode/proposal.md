## Why

当前信息条有三种显示模式（Always/Selected/Smart），但缺少一种经典 RTS 风格的模式——仅在选中或光标悬停时显示血条。这种模式在红色警戒等经典 RTS 中广泛使用，能让玩家专注于当前交互的单位，同时通过悬停快速查看任意单位状态。

此外，现有 Selected 模式也应支持悬停显示，提升操作效率。

## What Changes

- 新增 `Classic` 显示模式：仅选中或光标悬停的单位显示血条（不考虑受伤/经验变化）
- 修改 `Selected` 模式：在原有"仅选中"基础上，增加悬停显示血条
- 在 `unit_info_bar_system` 中增加悬停检测逻辑：查询光标世界坐标，与所有单位位置比较距离
- 更新 `InfoBarMode::next()` 循环顺序和默认模式
- `Ctrl+H` 循环：Always → Selected → Smart → Classic → Always

## Capabilities

### New Capabilities

无新增能力。

### Modified Capabilities

- `info-bar-display-modes`：新增 Classic 模式，修改 Selected 模式增加悬停支持

## Impact

- 影响 crate: `render_view/`（仅 `unit_info_bar.rs`）
- 需要访问 `Window` 和 `Camera` 进行光标世界坐标转换（已在 `selection.rs` 中有相同模式）
- 不引入新依赖
