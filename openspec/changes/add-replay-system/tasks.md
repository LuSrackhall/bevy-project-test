## 1. 修复仿真确定性 — 概率系统万分比改造

- [ ] 1.1 在 `simulation/src/types.rs` 中新增 `gen_probability_permyriad() -> u32` 方法，返回 0..10000 的确定性概率值
- [ ] 1.2 将 `combat/mod.rs` 中所有 `gen_probability()` 调用替换为 `gen_probability_permyriad()`，概率阈值从 f32 改为 u32 万分比
- [ ] 1.3 将 `combat/mod.rs` 中穿透概率（`pierce_chance: f32`）改为万分比整数，Arrow 组件字段类型同步修改
- [ ] 1.4 将 `combat/mod.rs` 中多重射击概率计算改为万分比整数运算
- [ ] 1.5 将 `combat/mod.rs` 中 Fisher-Yates 洗牌改为整数索引（`gen_probability_permyriad() * (i+1) / 10000`）
- [ ] 1.6 将 `combat/mod.rs` 中箭矢散布判定和建筑伤害计算改为万分比整数

## 2. 修复仿真确定性 — 比例计算整数化

- [ ] 2.1 将 `combat/mod.rs` 中骑兵闪避概率（`hp_cur as f32 / hp_max as f32`）改为万分比整数除法
- [ ] 2.2 将 `combat/mod.rs` 中吸血计算（`lifesteal_rate`）改为万分比整数乘除
- [ ] 2.3 将 `soldier/mod.rs` 中减速倍率 `powi()` 改为循环万分比乘法
- [ ] 2.4 将 `soldier/mod.rs` 中出生冷却（`60.0 / mult`）改为整数除法
- [ ] 2.5 将 `soldier/mod.rs` 中城市占领 HP、城市伤害、治疗量、升级门槛等比例计算改为万分比整数
- [ ] 2.6 将 `ai/mod.rs` 中 AI 攻击决策的 f32 比较（`ai_nearby as f32 > ... * 1.3`）改为整数比较

## 3. 修复仿真确定性 — 配置文件迁移

- [ ] 3.1 将 `content/combat.ron` 中所有 f32 概率/比率字段改为 u32 万分比
- [ ] 3.2 将 `content/soldier.ron` 中所有 f32 字段（`stack_mult`、`speed_penalty` 等）改为 u32 万分比
- [ ] 3.3 将 `content/city.ron` 中所有 f32 字段改为 u32 万分比
- [ ] 3.4 更新 config 解析代码（config.rs），将 f32 字段类型改为 u32
- [ ] 3.5 更新所有现有测试中的配置值和断言，匹配万分比新值

## 4. 修复仿真确定性 — HashMap 与 RNG

- [ ] 4.1 将 `combat/mod.rs` 中 nearest-enemy 扫描的 `HashMap<UnitId, ...>` 改为 `BTreeMap`
- [ ] 4.2 审查 simulation 层所有 HashMap 使用，确认无其他迭代顺序敏感场景
- [ ] 4.3 确认 `rand` crate 版本已锁定在 Cargo.lock 中

## 5. 黄金确定性测试

- [ ] 5.1 在 `simulation/src/lib.rs` 或新模块中实现 `hash_world_state(world: &World) -> u64` 函数，按 UnitId 排序遍历所有组件
- [ ] 5.2 编写黄金测试用例 1：空地图 + 无指令，1000 tick 后断言世界状态哈希一致
- [ ] 5.3 编写黄金测试用例 2：1v1 战斗 + 预定义指令，500 tick 后断言世界状态哈希一致
- [ ] 5.4 编写黄金测试用例 3：多城市混战 + AI + 混合指令，2000 tick 后断言世界状态哈希一致
- [ ] 5.5 确认 `cargo test -p simulation` 全部通过

## 6. Replay 数据结构与 serde

- [ ] 6.1 为 `Fixed`、`FixedVec2`、`UnitId` 添加 `#[derive(Serialize, Deserialize)]`
- [ ] 6.2 为 `SoldierType`、`ShieldState`、`Faction`、`SoldierState` 添加 serde derives
- [ ] 6.3 为 `SeekScope`、`SeekDirective`、`Action`、`GameCommand` 添加 serde derives
- [ ] 6.4 在 `simulation/src/types.rs` 中新增 `SimulationSeed(pub u64)` 资源
- [ ] 6.5 在 `init_simulation_world` 中插入 `SimulationSeed` 资源
- [ ] 6.6 创建 `simulation/src/replay.rs`，定义 `ReplayFile` 结构（含 format_version、seed、map_size、total_ticks、commands_per_tick: BTreeMap）
- [ ] 6.7 为 ReplayFile 编写序列化/反序列化单元测试

## 7. Replay 录制

- [ ] 7.1 在 `bevy_adapter` 中定义 `GameMode` 枚举（Live/Replay）和 `ReplayRecorder` 资源
- [ ] 7.2 在 `tick_driver_system` 中添加录制拦截点：提取命令后、注入 simulation 前，复制到 ReplayRecorder
- [ ] 7.3 确保仅录制外部玩家命令，不录制 AI 命令
- [ ] 7.4 实现 GameOver 时 ReplayRecorder.finish() 生成 ReplayFile 并写入磁盘
- [ ] 7.5 在 render_view 设置界面添加 "自动录制 Replay" 开关（默认开启）

## 8. Replay 回放

- [ ] 8.1 在 `bevy_adapter` 中定义 `ReplayController` 资源（replay_data、current_tick、target_tick、speed、is_paused、is_seeking）
- [ ] 8.2 实现 `replay_tick_driver_system`：从 ReplayFile 提取当前 tick 命令注入 simulation，调用 run_tick
- [ ] 8.3 通过 `run_if` 条件让 tick_driver_system 和 replay_tick_driver_system 互斥运行
- [ ] 8.4 实现暂停/继续功能
- [ ] 8.5 实现快进（2x/4x）：每帧执行多个 tick
- [ ] 8.6 实现 seek：从 tick 0 快速重放到目标 tick
- [ ] 8.7 在 bevy_adapter 中暴露 `ReplayStatus { is_replay, total_ticks }` 资源

## 9. Replay UI

- [ ] 9.1 在主菜单添加 "Load Replay" 按钮和文件选择逻辑
- [ ] 9.2 实现 Replay 播放器控制栏 UI（播放/暂停、1x/2x/4x、进度条、tick 计数器）
- [ ] 9.3 实现进度条拖拽 seek 交互
- [ ] 9.4 处理无效/不兼容 Replay 文件的错误提示

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/add-replay-system`
