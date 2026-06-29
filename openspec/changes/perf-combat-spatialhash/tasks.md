## 1. combat_engagement_system

- [x] 1.1 预构建 faction_map（从 all_units 拆出）
- [x] 1.2 构建 SpatialHash（cell_size=64）
- [x] 1.3 内层循环改用 spatial.query_nearby + faction_map
- [x] 1.4 seek_range > 3*cell_size 时 fallback 全量扫描
- [x] 1.5 运行 `cargo test -p simulation` 确认测试通过

## 2. melee_attack_system

- [x] 2.1 预构建 faction_map
- [x] 2.2 构建 SpatialHash（cell_size=32）
- [x] 2.3 内层循环改用 spatial.query_nearby + faction_map
- [x] 2.4 运行 `cargo test -p simulation` 确认测试通过

## 3. archer_attack_system

- [x] 3.1 预构建 faction_map
- [x] 3.2 构建 SpatialHash（cell_size=200）
- [x] 3.3 内层循环改用 spatial.query_nearby + faction_map
- [x] 3.4 运行 `cargo test -p simulation` 确认测试通过

## 4. arrow_movement_system

- [x] 4.1 预构建 soldier_faction_map 和 city_faction_map
- [x] 4.2 构建 SpatialHash（cell_size=32）
- [x] 4.3 碰撞检测改用 spatial.query_nearby + faction_map
- [x] 4.4 保持穿透（hit_units）和城市碰撞原有语义
- [x] 4.5 运行 `cargo test -p simulation` 确认测试通过

## 5. 最终验证

- [ ] 5.1 运行 `cargo test` 全项目确认无编译错误和测试失败

---

## Post-Implementation Workflow

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm
3. **Merge**: After user accepts, go to main branch and merge
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/perf-combat-spatialhash`
