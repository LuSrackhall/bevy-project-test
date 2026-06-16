## Context

Bevy 0.18 的 UI 交互系统有两个已知缺陷：
1. `Display::None` 切换到 `Display::Flex` 后，子节点的 `Interaction` 组件不会立即被正确初始化
2. `Changed<Interaction>` 在节点刚变为可见时可能不触发

当前索敌面板的下拉框和输入框都依赖这两个机制，导致交互失效。

## Goals / Non-Goals

**Goals:**
- 下拉框选项点击后能正确选中
- 范围输入框点击后能正确进入编辑模式并接收键盘输入
- 不引入第三方 UI 库（保持 Bevy 原生 UI）
- 性能无显著影响（避免每帧大量额外计算）

**Non-Goals:**
- 不修改 Bevy 引擎源码
- 不重构整个 HUD 系统
- 不改变功能行为（仅修复交互检测层）

## Decisions

### D1: 弹出层用位置偏移代替 Display 切换

下拉弹出层始终保持 `Display::Flex` + `Visibility::Visible`，通过 `Node.left` 控制位置：
- 关闭状态：`left: Val::Px(-9999.0)`（移出屏幕，不可见但仍参与布局计算）
- 打开状态：`left: Val::Px(0.0)`（正常位置）

子节点的 `Interaction` 组件始终正常初始化和更新，不受 Display 切换影响。

**性能影响：** 弹出层（4 个按钮）始终参与布局计算。对于 4 个简单按钮节点，开销可忽略不计（< 0.01ms/帧）。

### D2: 移除所有 Changed<Interaction> 依赖

所有 seek panel 交互系统改用 `Query<&Interaction, With<Marker>>`（无 `Changed` 过滤器），每帧检查 `Interaction::Pressed` 状态。

为防止持续按住鼠标导致重复触发，配合 `mouse.just_pressed(MouseButton::Left)` 做防抖：
- 仅在鼠标按下那一帧响应
- 不需要额外的标志位

### D3: 输入防重入

`seek_panel_input_system` 中，输入框的点击检测改为：
```rust
let input_pressed = input_btn.iter().any(|i| *i == Interaction::Pressed);
if input_pressed && mouse.just_pressed(MouseButton::Left) && !state.editing {
    state.editing = true;
    state.input_buffer = state.range_value.to_string();
}
```
`mouse.just_pressed` 确保只在按下那一帧触发一次，`!state.editing` 防止在编辑态中重复进入。

## Risks / Trade-offs

- **Risk:** 屏幕外偏移的节点仍参与布局计算，可能影响 Flexbox 布局
  → **Mitigation:** 弹出层使用 `PositionType::Absolute` 定位，不影响父容器的 Flex 布局

- **Trade-off:** 每帧检查 Interaction 状态比 Changed 事件多了几次比较操作
  → 开销极小（3 个 Query 各 1-4 个实体），无需优化
