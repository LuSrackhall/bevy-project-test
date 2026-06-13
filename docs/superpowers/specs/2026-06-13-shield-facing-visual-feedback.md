# 盾牌 & 朝向 & 掉落盾牌 视觉反馈设计

> **日期**: 2026-06-13
> **状态**: 待审批
> **范围**: render_view 层视觉反馈，simulation 层无需修改
> **依赖**: 盾牌系统、朝向系统已在 simulation 层实现完毕

---

## 1. 背景

simulation 层已完成以下系统，但 render_view 层没有对应的视觉反馈：
- 盾牌系统（ShieldItem、ShieldComponent、DroppedShield）
- 朝向系统（FacingDirection）

玩家无法观察到这些系统的运行效果。

---

## 2. 盾牌外观 — 持盾士兵身旁的小盾牌形状

### 2.1 实现位置

`crates/render_view/src/debug_shape.rs` 的 `draw_debug_shapes_system`

### 2.2 逻辑

遍历 simulation 世界中的士兵实体：
- 检查是否有 `ShieldComponent` 且 `ShieldItem.hp > 0`
- 有 → 在士兵圆形**右侧**绘制一个小盾牌形状
- 盾牌颜色：与士兵同色系但略浅

### 2.3 盾牌形状

用 Gizmos 画一个 **4x5 的小矩形**（宽4，高5），位于士兵圆形右侧偏移约 `collision_radius + 3` 像素。

颜色方案：
| 阵营 | 士兵颜色 | 盾牌颜色 |
|------|---------|---------|
| 玩家 | `rgb(0.3, 0.5, 0.9)` | `rgb(0.4, 0.6, 1.0)` |
| 敌人 | `rgb(0.9, 0.3, 0.3)` | `rgb(1.0, 0.4, 0.4)` |
| 中立 | `rgb(0.5, 0.5, 0.5)` | `rgb(0.6, 0.6, 0.6)` |

---

## 3. 白色盾牌血条 — HP 条上方

### 3.1 实现位置

`crates/render_view/src/unit_info_bar.rs` 的 `create_bar` 和 `update_bar`

### 3.2 布局变化

```
之前: Lv.X | HP条(绿/红) | EXP条(紫/蓝)
之后: Lv.X | 盾牌条(白/灰) | HP条(绿/红) | EXP条(紫/蓝)
```

### 3.3 盾牌条规格

| 属性 | 值 |
|------|-----|
| 宽度 | 40px |
| 高度 | 4px（比 HP 条略窄） |
| 背景色 | 深灰 `rgb(0.3, 0.3, 0.3)` |
| 填充色 | 白色 `rgb(0.9, 0.9, 0.9)` |
| 数值文本 | "盾牌 cur/max" |
| 显示条件 | 仅持有盾牌的单位（有 `ShieldComponent` + `ShieldItem`） |

### 3.4 位置偏移

- 盾牌条 Y 偏移：比 HP 条高 6px（`BAR_OFFSET_Y + 6`）
- 盾牌条数值文本：与盾牌条同行右侧

---

## 4. 朝向线段 — 从圆心向外

### 4.1 实现位置

`crates/render_view/src/debug_shape.rs` 的 `draw_debug_shapes_system`

### 4.2 逻辑

遍历 simulation 世界中的士兵实体：
- 读取 `FacingDirection.angle`
- 从士兵位置出发，沿朝向方向画一条线段
- 线段长度 = `collision_radius * 1.5`
- 线段颜色：与士兵同色但更亮（白色调）

### 4.3 颜色方案

| 阵营 | 线段颜色 |
|------|---------|
| 玩家 | `rgb(0.5, 0.7, 1.0)` |
| 敌人 | `rgb(1.0, 0.5, 0.5)` |
| 中立 | `rgb(0.7, 0.7, 0.7)` |

### 4.4 角度转换

simulation 层的 `FacingDirection.angle` 是定点数角度（0-360°），需要：
1. `angle.to_float()` 转为 f32
2. 转为弧度：`angle_rad = angle_deg * PI / 180.0`
3. 计算线段终点：`end = pos + Vec2(angle_rad.cos(), angle_rad.sin()) * length`

---

## 5. 掉落盾牌 — 地面灰色盾牌

### 5.1 实现位置

`crates/render_view/src/debug_shape.rs` 新增 `draw_dropped_shields_system`

### 5.2 逻辑

遍历 simulation 世界中的 `DroppedShield` 实体：
- 读取 `position`
- 在该位置绘制灰色盾牌形状

### 5.3 盾牌形状

与士兵身旁盾牌形状一致（矩形），但稍大：
- 尺寸：**6x8 像素**
- 颜色：灰色 `rgb(0.6, 0.6, 0.6)`

### 5.4 可选增强

悬停时显示盾牌 HP 数值（需要在 selection 系统中处理 DroppedShield 的悬停检测）。

---

## 6. 实现优先级

| 优先级 | 任务 | 说明 |
|--------|------|------|
| P0 | 朝向线段 | 最简单，只需在 debug_shape 加几行 |
| P0 | 盾牌外观 | debug_shape 加盾牌矩形 |
| P0 | 掉落盾牌渲染 | debug_shape 新增系统 |
| P1 | 盾牌血条 | unit_info_bar 需要修改布局 |

---

## 7. 架构约束

- 所有变更在 `render_view` 层，不修改 `simulation`
- `render_view` 通过 `SimulationWorld.0` 读取 simulation 组件
- 不使用 `PresentationPosition`（当前架构下直接读 LogicalPosition）
- 盾牌和朝向信息通过 simulation 世界的 ECS 组件暴露，render_view 直接查询
