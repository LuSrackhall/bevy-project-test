## Why

联机主菜单中 `NetworkPlayerCount` 和 `NetworkPlayerId` 两个选择器按钮为静态显示，用户无法选择 3-4 玩家配置。同时 `开始联机` 按钮的 Query 存在预存 Bug（tuple query 无法跨兄弟实体匹配），导致所有网络值始终回退到默认值。这两个问题阻碍了 3+ 玩家多人游戏的可用性。

## What Changes

- `NetworkPlayerCount` 按钮添加 `.observe()`：点击循环 `2 → 3 → 4 → 2`，同步更新子 Text 显示
- `NetworkPlayerId` 按钮添加 `.observe()`：点击循环 `0 → 1 → … → (count-1) → 0`，同步更新子 Text 显示
- Count 减小时自动 clamp ID（Count observer 通过跨实体 Query 写入 ID 组件和 ID Text）
- 修复 `开始联机` 按钮 Query：拆为三个独立的 `Query<&T>`，分别读取每项配置

## Capabilities

### New Capabilities
- `multiplayer-ui-config`: 主菜单联机区域的玩家数量（2-4）和玩家序号（0..count-1）交互式选择器

### Modified Capabilities
<!-- No existing spec requirements change. This is a pure UI addition. -->

## Impact

- `crates/render_view/src/ui/menu.rs`：给两个按钮添加 observer，修改开始按钮查询逻辑
- 无 simulation / bevy_adapter / relay 层改动
- 无新增文件、资源或系统
