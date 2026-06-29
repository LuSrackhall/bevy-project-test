# Performance Guide

## Scaling Thresholds

| Entity Count | Expected Bottleneck | Mitigation Status |
|---|---|---|
| ~1,000 | Combat SpatialHash rebuilds, overlap iterations | DONE (SpatialHash + overlap reuse) |
| ~5,000 | 17 full table scans per tick (~700k component reads) | IN PROGRESS (build_soldier_index dedup) |
| ~10,000 | Full table scans dominate; dirty-flag partitioning needed | PLANNED |
| ~50,000 | Arrow.hit_units allocation pressure; sequential phase bottleneck | PLANNED |
| ~100,000 | ECS archetype cache misses (14-wide rows for 5-column queries) | PLANNED (SoA projection) |
| ~1,000,000 | Entity count + indirect UnitId lookups | PLANNED (chunk-based spatial) |

## Optimization History

### Round 1: UnitIdEntityIndex + Selection (2026-06-28)
- `UnitIdEntityIndex(HashMap<UnitId, Entity>)` for O(1) lookups
- SpatialHash HashMap→BTreeMap for determinism
- selection_visual_system O(m*n)→O(m)
- HUD query O(m*n)→O(m)

### Round 2: Combat SpatialHash (2026-06-29)
- 4 combat systems promoted to SpatialHash O(n*k)
- combat_engagement (cell_size=64), melee (32), archer (200), arrow (32)
- BTreeMap + sorted-by-UnitId Vec for determinism

### Round 3: Structural Fixes + Profiling (2026-06-29)
- combat_engagement O(S²) Vec::find → HashMap O(1)
- overlap_resolution SpatialHash reuse (3 builds→1)
- build_soldier_index helper (12+ HashMap constructions → shared function)
- UnitIdEntityIndex incremental (spawn/despawn updates)
- SpatialHash query_range interface
- bevy_adapter tracing instrumentation (feature-gated)
- render_view debug_render feature gate
- Criterion benchmark crate (crates/bench/)

## Constitution Tier 2 Triggers

Tier 2 clauses activate when ANY condition is met:
1. Simulation tick time > 2ms
2. Any single hot system > 30% of tick budget
3. Active entity count > 1,000

## Measurement Methodology

### Tracy Profiling (bevy_adapter)
Run with `--features tracing` to enable Tracy integration:
```bash
cargo run --features bevy_adapter/tracing
```
Connect Tracy client to see per-tick spans.

### Criterion Benchmarks (crates/bench/)
```bash
cargo bench -p bench
```
Results in `target/criterion/`.

### Per-Phase Breakdown
Use `phase_bench` to identify which system dominates:
```bash
cargo bench -p bench --bench phase_bench
```
