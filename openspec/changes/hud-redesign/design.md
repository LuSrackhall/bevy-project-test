## Context

当前 `hud.rs` 约 305 行，资源追踪用 `HudTexts` struct 持有各文本实体 ID。底部面板仅支持城池选中，通过 `SelectedCity` 资源驱动。选中兵士通过 `SelectionState.selected_unit_ids` 追踪但无 UI 展示。

## Goals / Non-Goals

**Goals:**
- 底部三区分明：左(选中信息) / 中(命令) / 右(小地图+图鉴)
- 单选士兵展示完整属性（含特殊效果文案）
- 多选聚合展示
- 城池面板增强
- 兵种按钮悬停 → 右区图鉴切换
- 拆分 hud.rs 为多模块

**Non-Goals:**
- 不做美术贴图/图标
- 小地图不做实际渲染（框架占位）

## Decisions

### D1: 文件拆分

```
crates/render_view/src/ui/
  hud.rs            ← 骨架布局 + 顶栏 + 资源定义
  soldier_panel.rs  ← 士兵信息面板
  city_panel.rs     ← 城池信息面板
  command_card.rs   ← 命令卡片
  compendium.rs     ← 兵种图鉴
```

每个模块含 setup_* 和 update_* 函数，由 hud.rs 中的 HudTexts 统一持有各文本实体 ID。

### D2: HudTexts 扩展

新增字段覆盖三个区的所有动态文本+节点实体：
```
soldier_panel_root / city_panel_root / command_card_root / compendium_root
→ 每个 root 下有对应的 text/bar/button 子实体 ID
```

### D3: 兵种特殊效果数据

不在 simulation 层存储描述文本（违反架构宪法：simulation 不包含 UI 内容）。在 render_view 层维护一个 `UnitDescriptions` 资源：

```rust
struct UnitDescriptions {
    militia_effect: String,    // "无特殊效果"
    militia_desc: String,      // "基础步兵，成本低廉..."
    infantry_effect: String,   // "举盾：可举盾大幅减伤"
    infantry_desc: String,
    archer_effect: String,     // "远程+穿透：箭矢可穿透敌人"
    archer_desc: String,
    cavalry_effect: String,    // "闪避+无畏：受伤时可闪避"
    cavalry_desc: String,
}
```

### D4: 悬停检测

Bezy 的 `Interaction` 组件只支持 `None/Hovered/Pressed`。城池兵种按钮悬停时发送事件或在 update 中检测 `Interaction::Hovered`，更新右区图鉴内容。

### D5: 选中数据流向

```
SelectionState.selected_unit_ids
    ↓
  soldier_panel::update() 读取 sim_world → 查对应实体 → 渲染属性
```

多选时聚合：按 SoldierTypeComponent 分组计数，总 HP 累加。
