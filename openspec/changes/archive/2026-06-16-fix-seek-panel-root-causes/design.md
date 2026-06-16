## Context

通过深入分析 Bevy 0.18 源码（`focus.rs`、`picking_backend.rs`），发现索敌面板的 6 个根因 bug。

## Decisions

### D1: 弹出层用 Display::None（不用 left 偏移）

Bevy 拾取管线在 `picking_backend.rs:186` 检查 `node.node.size() == Vec2::ZERO` 跳过 `Display::None` 节点，在 `focus.rs:249-257` 跳过隐藏可见性节点并重置 `Interaction`。`Display::None` 是 Bevy 原生支持的隐藏方式，`left: -9999` 会导致 Taffy 布局引擎在极端偏移→正常位置转换时产生边界计算错误。

### D2: 存储 Text 子实体 ID

`setup_hud` 中 `.with_child((Text::new(...), ...)).id()` 返回的是 **Button 父实体** 的 ID，不是 Text 子实体。后续 `Query<&mut Text>` 查询该 ID 会静默失败。改为在 `with_children` 闭包内捕获子实体 ID：

```rust
let mut text_entity = Entity::PLACEHOLDER;
p.spawn((Button, ...)).with_children(|p| {
    text_entity = p.spawn((Text::new(...), ...)).id();
});
ht.seek_scope_text = Some(text_entity);
```

### D3: selection_click_system UI 守卫

`selection_click_system` 在每次左键点击时执行，如果点击位置没有单位/城池就清空选区。点击 UI 元素（如输入框）也会触发清空，导致编辑态被破坏。修复：查询所有 `Interaction != None` 的实体，如果有则跳过选择逻辑。

### D4: seek_panel_mode_system 编辑态保护

模式切换系统在选区变化时重置 `editing` 和 `input_buffer`。如果用户正在编辑输入框时选区被清空（D3 的问题），编辑态会被重置。修复：编辑态下跳过模式切换。

### D5: 系统排序

`lib.rs` 和 `ui/mod.rs` 的 `add_systems` 调用之间无顺序保证。HUD 系统应在选择系统之前运行，确保 UI 交互先被处理。
