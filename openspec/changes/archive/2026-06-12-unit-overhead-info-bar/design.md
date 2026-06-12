## Context

当前 `render_view` crate 通过 `debug_shape.rs` 绘制单位几何图形，通过 `ui/hud.rs` 在屏幕空间底部面板显示选中单位的详细信息。项目已使用 `bevy_prototype_lyon` 进行矢量图形绘制，Bevy 的 `Text2D` 组件可直接用于世界空间文字渲染。

玩家需要在不点击单位的情况下快速扫视战场获取信息——这是 RTS 品类的基础体验需求。

约束：
- 必须遵循 CLAUDE.md 架构宪法——不在 simulation 层添加渲染代码，不反向依赖
- 只使用已有依赖（`bevy_prototype_lyon`、`Text2D`），不引入新 crate
- 实体查找必须 O(1)，不用嵌套 Query

## Goals / Non-Goals

**Goals:**
- 在士兵和城市头顶以世界空间绘制等级数字、血量条、经验条
- 提供三种显示模式（Always/Selected/Smart），支持 Ctrl+H 循环切换
- 精简底部面板，移除已上浮到头顶的血量、等级、经验显示

**Non-Goals:**
- 不创建设置面板 UI（仅快捷键切换）
- 不支持自定义条颜色/尺寸（固定配置）
- 不修改 simulation/bevy_adapter/presentation 层
- 不做动画过渡（条宽度直接变化，不插值）

## Decisions

### 1. 世界空间 vs 屏幕空间
**选择：世界空间。** 信息条作为子实体锚定在单位 `GlobalTransform` 下，随缩放自然变化。屏幕空间方案需要每帧做 WorldToScreen 投影，增加复杂度且违反 RTS 惯例。

### 2. ShapeBundle + Text2D vs Gizmos
**选择：ShapeBundle + Text2D。** Gizmos 适合调试图形但不支持文字叠加。ShapeBundle（来自 `bevy_prototype_lyon`）创建持久实体，支持层级关系和可见性控制；Text2D 直接在世界空间渲染文字。两者都已存在于项目依赖中。

替代方案：使用 Bevy UI Node 世界空间模式——需要额外插件（如 `bevy_ui_world_space`），引入新依赖，放弃。

### 3. 实体生命周期管理
**选择：子实体模式。** 信息条（血条矩形、经验条矩形、等级文字、数值文字）作为父单位的子实体创建。父实体销毁时 Bevy 自动清理子实体。首次检测到满足条件的实体时创建信息条，挂载 `UnitInfoBar` 标记避免重复创建。

替代方案：每帧重建——简单但产生大量实体创建/销毁开销，放弃。

### 4. 显示模式存储
**选择：Resource。** `UnitInfoBarSettings` 作为 Bevy Resource 存储当前模式，全局共享。快捷键系统读取并修改此资源，渲染系统读取它来决定显示行为。

### 5. 底部面板精简策略
**选择：仅移除字段更新逻辑。** 不在 `hud.rs` 中删除 UI 节点（避免影响面板布局结构），只在 `update_bottom_panel` 中移除对 Health/Level 的文本更新调用。面板布局保持不变，被移除的字段留空。

## Risks / Trade-offs

- **性能风险：大量单位时信息条实体数量膨胀。** → 每个单位创建 4-5 个子实体（血条底+填充、经验条底+填充、等级文字、数值文字），1000 单位 = 5000+ 实体。缓解：Smart 模式下大部分单位不显示信息条子实体；未来可优化为单实体多形状批量绘制。
- **快捷键冲突。** → `Ctrl+H` 当前未使用，但未来可能冲突。缓解：在 `lib.rs` 中注册快捷键时添加注释标记占用。
- **世界空间条极远时不清晰。** → 这是世界空间的固有特性。缓解：可选增加最小/最大缩放范围限制（Non-Goal，未来可做）。
