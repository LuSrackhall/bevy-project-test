## 1. 配置层变更

- [x] 1.1 创建 `content/map/small.ron`、`medium.ron`、`large.ron`、`huge.ron` 四个配置文件
- [x] 1.2 修改 `MapGenConfig`：新增 `city_density: u32` 字段，`neutral_city_ratio` 从 `[f32; 2]` 改为 `[u32; 2]`
- [x] 1.3 删除旧 `content/map.ron`

## 2. Simulation 层变更

- [x] 2.1 在 `simulation/src/map/mod.rs` 中定义 `MapSize` 枚举（Small/Medium/Large/Huge）
- [x] 2.2 修改 `generate_map` 签名为 `generate_map(world, map_size: MapSize)`
- [x] 2.3 实现密度驱动城池数量计算
- [x] 2.4 修复 `neutral_city_ratio` 的 f32 运算为整数运算
- [x] 2.5 `generate_map` 根据 MapSize 加载对应配置文件（include_str!）
- [x] 2.6 修复所有现有测试

## 3. Bevy Adapter 层变更

- [x] 3.1 定义 `MapBounds { width: f32, height: f32 }` 资源
- [x] 3.2 在 `reset_game_system` 完成后创建 `MapBounds` 资源
- [x] 3.3 `NeedsGameReset` 从 `bool` 改为枚举（None/SameSize/NewGame(MapSize)）
- [x] 3.4 保存当前 `MapSize` 到资源（供 SameSize 使用）

## 4. 镜头系统重写

- [x] 4.1 实现动态缩放范围（基于 MapBounds + 窗口尺寸）
- [x] 4.2 修改拖拽为仅中键，速度 = delta * ortho.scale
- [x] 4.3 实现镜头边界限制（padding = map_size * 0.05，min 100）
- [x] 4.4 移除右键拖拽逻辑

## 5. UI 变更

- [x] 5.1 主菜单增加 4 个地图大小按钮
- [x] 5.2 按钮设 `NeedsGameReset::NewGame(MapSize)` + `NextState(Playing)`
- [x] 5.3 更新暂停/GameOver 按钮使用 `NeedsGameReset::SameSize`
- [x] 5.4 更新 `reset_game_system` 处理 NeedsGameReset 枚举

## 6. 编译与验证

- [x] 6.1 编译通过（`cargo build`）
- [x] 6.2 所有现有测试通过（`cargo test`）
- [x] 6.3 手动验证：4 种地图大小都能正常生成
- [x] 6.4 手动验证：镜头缩放范围随地图大小变化
- [x] 6.5 手动验证：中键拖拽自适应速度
- [x] 6.6 手动验证：镜头不超出地图边界
- [x] 6.7 手动验证：右键不触发拖拽

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/add-map-size-camera`
