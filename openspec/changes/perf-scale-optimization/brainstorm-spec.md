# perf-scale-optimization: 性能规模化优化

## Context

当前项目在 1000 单位规模下出现持续帧率下降。经过代码审查发现三个结构性性能缺陷：combat_engagement_system 的 O(S²) 线性扫描、overlap_resolution_system 每 tick 3 次 SpatialHash 重建、每 tick 12+ 次冗余 HashMap 全表构建。同时缺少 profiling 基础设施。

**实施结果**：107 测试通过。性能从 1000 单位提升到 ~1300 单位，但仍远未达到 10 万~100 万目标。瓶颈大概率在渲染层（draw_debug_shapes_system 每帧画千级 Gizmos、unit_info_bar_system 每帧更新 7N 子实体），而非 simulation 层。

## Goals / Non-Goals

**Goals:**
- 修复 simulation 中的结构性 O(n²) 算法缺陷
- 消除每 tick 12+ 次冗余全表扫描
- 建立 profiling 基础设施（tracy + criterion）
- 补齐宪法合规缺失项（§4.3 复杂度声明、docs/performance.md）
- 为 10k/100k/1M 单位扩展建立可测量的基线

**Non-Goals:**
- 引入多线程并行（宪法 §13.5 要求确定性优先，当前阶段顺序执行足够）
- 替换 bevy_ecs 为自定义 ECS
- SoA 组件布局（100k+ 才需要，当前过早）
- 渲染层大规模重构（instanced rendering、shader-based UI bars）

## Decisions

### D1: 优化先于 Profiling

**决策**：先修结构性 bug（Phase 1-3），再建 profiling（Phase 4-5）。

**理由**：代码审查已确认三个结构性瓶颈，不需要 profiling 验证：
1. `combat_engagement_system:142` 的 `Vec::find` O(S²) 线性扫描
2. `overlap_resolution_system` 每 tick 3 次 SpatialHash 重建
3. 12+ 次冗余 HashMap 全表构建

Profiling 用于确认后续优化方向，不是用来发现已知问题。

### D2: 保持 BTreeMap + sorted Vec（禁止 HashMap 替代）

**决策**：SpatialHash 内部存储保持 `BTreeMap<(i32,i32), Vec<SpatialEntry>>`，cell 内 Vec 保持按 UnitId 排序插入。

**理由**：
- BTreeMap 保证 cell 遍历顺序确定（字典序）
- sorted Vec 保证同 cell 内迭代顺序确定（UnitId 序）
- HashMap 替代会导致等距单位的攻击目标选择不一致，一个不同的 dodge roll 会导致完整模拟分歧

**评审确认**：Round 2 Agent 2（正确性）给出了具体的 breakage scenario。

### D3: 禁止跨系统共享 HashMap 资源

**决策**：提取 `build_soldier_index(world)` 辅助函数消除代码重复，但每个系统独立调用，不作为 Resource 跨阶段共享。

**理由**：melee_attack_system 可以杀死 entity，导致后续 arrow_movement_system 读到过期 Entity handles。保守正确路径是每个系统独立构建快照。

**评审确认**：Round 4 Agent 2。

### D4: UnitIdMapper 和 UnitIdEntityIndex 保持分离

**决策**：
- `UnitIdEntityIndex`（simulation）：增量更新，spawn/despawn 时维护
- `UnitIdMapper`（bevy_adapter）：render 侧双向映射，保持独立

**理由**：宪法层拓扑 simulation ← bevy_adapter，simulation 不能依赖 bevy_adapter。两个索引各服务其层。

**评审确认**：Round 4 Agent 2 明确 REJECTED 合并方案。

### D5: query_range 泛化接口 + 各系统独立 cell_size

**决策**：新增 `SpatialHash::query_range(pos, radius)` 方法，替代硬编码 3×3 邻域的 `query_nearby`。各系统保持独立 cell_size（melee=32, archer=200, engagement=64）。

**理由**：
- `query_nearby` 硬编码 `for dx in -1..=1`，cell_size=32 时弓箭手只搜 64 范围，会静默丢失目标（Round 3 Agent 2 发现的正确性 bug）
- 各系统保持独立 cell_size 是"best of both worlds"（Round 4 Agent 2）
- BTreeMap 下 49 cell 查询仍是确定性的

### D6: simulation 零 profiling 依赖

**决策**：simulation crate 不引入 tracing、tracy 或任何 profiling 依赖。所有 instrumentation 在 bevy_adapter 层完成。

**理由**：
- `tracing::Span` 内部调用 `Instant::now()`，泄漏非确定性到 simulation（Round 2 Agent 1 发现的 §2.6 违规）
- bevy_ecs 白名单是 allowlist，tracing 不在其中
- simulation 提供纯函数 `run_tick()`，profiling 在调用侧完成

### D7: 独立 bench crate（§21 合规）

**决策**：创建 `crates/bench/` 独立 binary crate，依赖 simulation library。不在 simulation 上加 feature flag。

**理由**：宪法 §21 要求 benchmark 是独立 binary scope，不允许在 simulation 中 conditionally compile。

## Phases

### Phase 1: 结构性 Bug 修复

**1a. combat_engagement_system O(S²) → O(S)**
- 文件：`crates/simulation/src/combat/mod.rs:142`
- 当前：`soldiers.iter().find(|(id, ..)| *id == suid)` 每 soldier 线性扫描
- 修复：soldiers 改为 HashMap<UnitId, (Entity, ...)>，用 `.get()` 查找
- 外层循环 `sorted_soldier_uids` 保持排序，确定性不受影响
- 复杂度：O(S²) → O(S)

**1b. 提取 build_soldier_index 辅助函数**
- 消除 12+ 次重复的 HashMap 构建代码
- 各系统独立调用（不共享 Resource，见 D3）
- 函数签名：`fn build_soldier_index(world: &mut World) -> HashMap<UnitId, SoldierSnapshot>`

**1c. overlap_resolution SpatialHash 迭代间复用**
- 文件：`crates/simulation/src/soldier/mod.rs:441-517`
- 当前：max_iterations=3，每次迭代重建 SpatialHash
- 修复：迭代间复用同一 SpatialHash，仅在位置变化时增量更新
- 预期效果：3 次构建 → 1 次

### Phase 2: SpatialHash 统一 + query_range

**2a. 新增 query_range(pos, radius) 接口**
- 泛化 cell sweep：`(2 * ceil(radius / cell_size) + 1)^2` cells
- 替代硬编码 3×3 的 query_nearby
- 保持 BTreeMap 确定性遍历

**2b. 各系统独立 cell_size 保持**
- melee_attack: cell_size=32（范围 30）
- combat_engagement: cell_size=64（范围 60-90）
- archer_attack: cell_size=200（范围 200）
- arrow_movement: cell_size=32
- 不做统一 cell_size（避免弓箭手查询成本 5.4x 增加）

### Phase 3: UnitIdEntityIndex 增量化

- 当前：每 tick O(N) 全量重建
- 修复：spawn 时 insert，despawn 时 remove
- 成本：O(N) → O(changed_per_tick)
- UnitIdMapper（bevy_adapter）保持独立（见 D4）

### Phase 4: Profiling 基础设施

**4a. bevy_adapter tracing 插桩**
- 围绕 `run_tick_default()` 添加 `tracing::info_span!("tick")`
- 注册 `tracing-tracy` subscriber（feature-gated）
- simulation 零改动（见 D6）

**4b. render_view debug_render feature gate**
- `draw_debug_shapes_system` 和 `unit_info_bar_system` 加 `#[cfg(feature = "debug_render")]`
- feature 名称匹配宪法 §21

### Phase 5: Benchmark Crate + CI

**5a. crates/bench/ 独立 binary**
- criterion benchmarks
- 场景：空世界、1k 空闲、1k 对战、10k 对战
- 完整 tick + 单 phase 耗时

**5b. CI 回归门**
- `cargo bench` + baseline 比较
- 5% 回归 = 失败

### Phase 6: 合规补齐

**6a. §4.3 复杂度声明**
- 所有 hot system 添加 Complexity/Memory/Hot-Path doc-comments

**6b. docs/performance.md**
- 宪法引用但不存在的文档
- 记录性能基线、优化历史、scaling 阈值

**6c. ADR-004/005**
- ADR-004: 持久化 SpatialIndex vs 每 tick 重建决策
- ADR-005: Phase 依赖图与未来并行策略

## Risks / Trade-offs

| 风险 | 影响 | 缓解 |
|------|------|------|
| Phase 1b 辅助函数仍需每系统构建 HashMap | 不如共享 Resource 高效 | 正确性优先；构建成本 O(N) 远小于 O(N²) bug |
| query_range 多 cell 查询成本（archer 49 cells） | 比当前 archer 独立 SpatialHash 慢 | 各系统保持独立 cell_size，archer 用 200 cell_size |
| UnitIdEntityIndex 增量化遗漏某些 despawn 路径 | 索引与 World 不一致 | 保留 `world.get_entity().is_ok()` 校验作为安全网 |
| bevy_adapter tracing 仅记录 tick 级别 | 无法看到 simulation 内部各 phase 耗时 | criterion bench crate 提供 phase 级别精度 |

## Scaling Thresholds（供未来参考）

| 阈值 | 预期瓶颈 | 对策 |
|------|----------|------|
| ~5k | overlap_resolution 迭代 + 全表扫描 | Phase 1-2 修复后应可支撑 |
| ~10k | 17 次全表扫描 × 14 组件 = 2.4M 读取 | dirty-flag 分区（active/moved/changed） |
| ~50k | Arrow.hit_units Vec 无限分配 | 阶段并行 + 专用碰撞结构 |
| ~100k | ECS archetype 宽行 L2/L3 缓存未命中 | SoA 投影（CombatSlice） |
| ~1M | Entity 数量本身 + UnitId 间接查找 | chunk-based 空间分区 |
