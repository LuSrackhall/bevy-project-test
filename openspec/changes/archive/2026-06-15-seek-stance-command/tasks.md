## 1. Content 层 — 配置文件清理

- [x] 1.1 删除 `content/units.ron` 中 4 个兵种的 `aggression_range` 行
- [x] 1.2 更新 `content/units.ron` 文件头注释，移除 `aggression_range` 字段说明，新增 `SeekStance` 命令相关说明

## 2. Simulation 层 — 类型与配置

- [x] 2.1 在 `simulation/src/command.rs` 中新增 `SeekScope` 枚举（`All`, `ByType(SoldierType)`）和 `SeekDirective` 结构体（`scope`, `seek_range`, `issue_tick`）
- [x] 2.2 在 `simulation/src/command.rs` 中新增 `Action::SetSeekStance` 变体（`scope`, `seek_range`, `unit_ids`）
- [x] 2.3 在 `simulation/src/command.rs` 中新增 `GlobalSeekDirective` 资源（`Vec<SeekDirective>`）
- [x] 2.4 从 `simulation/src/soldier/config.rs` 的 `SoldierUnitConfig` 中移除 `aggression_range` 字段，添加 `#[serde(default)]` 属性确保向后兼容
- [x] 2.5 移除 `SoldierUnitConfig` 中 `aggression_range` 的 serde default 函数（若有）

## 3. Simulation 层 — SeekStance 组件与核心逻辑

- [x] 3.1 在 `simulation/src/soldier/mod.rs` 中新增 `SeekStance` 组件（`active: bool`, `seek_range: u32`），默认 `{ false, 0 }`
- [x] 3.2 修改 `city_spawn_system`：新生成单位继承 `GlobalSeekDirective` 中的索敌指令，设置对应的 `SeekStance`
- [x] 3.3 修改 `combat_engagement_system`：用 `SeekStance` 替换 `aggression_range`，仅在 `active && 敌人在 seek_range 内` 时设置 `Movement.target`
- [x] 3.4 在 `consume_commands_system` 中新增 `Action::SetSeekStance` 消费分支，处理 All/ByType/unit_ids 三种粒度

## 4. Simulation 层 — 测试

- [x] 4.1 新增 `SeekStance` 默认值单元测试
- [x] 4.2 新增 `GlobalSeekDirective` 继承逻辑测试（新生成单位正确继承全局指令）
- [x] 4.3 新增 `consume_commands_system` 消费 `SetSeekStance` 的测试（All / ByType / unit_ids）
- [x] 4.4 新增 `combat_engagement_system` 基于 `SeekStance` 的行为测试（索敌关闭不移动 / 索敌开启移动 / force_move 跳过）
- [x] 4.5 验证移除 `aggression_range` 后旧配置向后兼容（缺失字段不报错）

## 5. Render View 层 — UI 入口

- [x] 5.1 在 `render_view/src/selection.rs` 中新增 `seek_stance_shortcut_system`（S 键对选中单位下发索敌命令，默认范围 30）
- [x] 5.2 新增全局索敌 UI 面板（toolbar 中添加全体开/步兵/弓兵/骑兵/全关按钮 + 索敌状态显示），生成 `Action::SetSeekStance` 命令

## 6. 集成验证

- [x] 6.1 `cargo test -p simulation` 全部通过（39/39）
- [x] 6.2 `cargo build` 全项目编译通过
- [ ] 6.3 手动验证：选中单位后不自动索敌，下发索敌命令后单位在范围内主动移动
- [ ] 6.4 手动验证：全局索敌面板对新生成的单位也生效
