## Verification Report: add-map-size-camera

### Completeness

| 检查项 | 状态 |
|--------|------|
| tasks.md 完成率 | 28/28 (100%) |
| map-presets spec 需求覆盖 | 5/5 |
| camera-improvements spec 需求覆盖 | 4/4 |
| simulation-crate spec 需求覆盖 | 1/1 |
| bevy-adapter-crate spec 需求覆盖 | 1/1 |
| render-view-crate spec 需求覆盖 | 2/2 |
| game-lifecycle spec 需求覆盖 | 1/1 |

### Correctness

| 需求 | 实现证据 | 状态 |
|------|----------|------|
| MapSize 枚举 | `simulation/src/map/mod.rs:22` | PASS |
| 密度驱动城池数 | `simulation/src/map/mod.rs:43` | PASS |
| 配置文件拆分 | `content/map/{small,medium,large,huge}.ron` | PASS |
| neutral_city_ratio u32 | `map/config.rs:16` — `[u32; 2]` | PASS |
| NeedsGameReset 枚举 | `render_view/src/lib.rs:25` | PASS |
| 主菜单地图选择 | `render_view/src/ui/menu.rs:55` — 4 按钮 | PASS |
| 动态缩放范围 | `camera.rs:136` — 6x map size | PASS |
| 光标居中缩放 | `camera.rs:147-153` | PASS |
| 中键拖拽 | `camera.rs:55` | PASS |
| 边缘滚动 | `camera.rs:79` | PASS |
| 边界墙可视化 | `debug_shape.rs` — 红色矩形 | PASS |
| 单位不可穿越 | `soldier/mod.rs` — WallBounds clamp | PASS |
| MapBounds 桥接 | `bevy_adapter/src/lib.rs:22` | PASS |
| 全屏启动 | `main.rs` — BorderlessFullscreen | PASS |

### Coherence

| 设计决策 | 实现一致性 | 备注 |
|----------|-----------|------|
| NeedsGameReset 枚举 | PASS | None/SameSize/NewGame 三变体 |
| 密度驱动 | PASS | area / density ± 15% |
| MapBounds 在 bevy_adapter | PASS | 纯数据翻译 |
| 边界墙 2x 地图 | PASS | WallBounds ± half map |
| 缩放 0.6% 乘法步长 | PASS | 0.006 per scroll |

### 额外实现（设计文档未覆盖但合理）

- 边缘滚动（设计时讨论过，用户确认需要）
- 光标居中缩放（zoom toward cursor）
- 边界墙系统（2x map size，红色线条，单位不可穿越）
- 全屏默认启动

### 结论

所有需求已正确实现。编译通过，83 个测试通过，手动验证通过。
