## 1. 战斗系统修复 (Combat Fixes)

- [ ] 1.1 修复 archer_attack_system 多重射击目标选择：收集攻击范围内所有敌方单位，按距离排序取前 N 个，每支 Arrow 射向不同目标
- [ ] 1.2 为 Arrow Entity 附加可见的 Lyon Shape（圆形半径 3px，颜色按阵营），实现箭矢视觉渲染
- [ ] 1.3 修复 arrow_damage_system 减速 debuff 叠加逻辑：已有 SlowDebuff 时 stacks+1（上限保证移速 ≥ 35%），刷新 timer；无则插入 stacks=1
- [ ] 1.4 新增 slow_debuff_tick_system：每帧 tick SlowDebuff timer，到期后移除 Component，注册到 SoldierPlugin
- [ ] 1.5 修复 melee_attack_system 攻击频率 bug：tick attacker 的 attack_timer，仅在 just_finished() 时执行攻击
- [ ] 1.6 修复 melee_attack_system 吸血效果：无畏状态骑兵额外 +15% 吸血率，与等级吸血叠加

## 2. 士兵系统修复 (Soldier Fixes)

- [ ] 2.1 解耦 soldier_city_interaction_system：移除内嵌的占领逻辑（翻转 faction 等），只负责扣血/回血/升级贡献 + despawn 士兵。占领判定完全交给 city_capture_system
- [ ] 2.2 确保 city_capture_system 正确处理攻城翻转后的流亡士兵标记（is_exiled = true），流亡士兵不 despawn
- [ ] 2.3 在 City struct 中新增 spawn_direction: Vec2 字段，产兵时确定方向（朝向最近敌方/中立城池，无目标默认 Vec2::X）
- [ ] 2.4 修复 soldier_aura_heal_system：光环出兵走廊方向从 city.spawn_direction 读取，替换硬编码 Vec2::X

## 3. 城池交互修复 (City Interaction)

- [ ] 3.1 新增 city_click_system：Playing 状态下监测左键点击（桌面）/单指触摸（移动端），世界坐标命中城池 visual_radius 时发出 CitySelectedEvent（仅己方城池）
- [ ] 3.2 新增 city_visual_update_system：城池 faction 或 level 变化时更新 Lyon Fill 颜色和圆环半径
- [ ] 3.3 修复 city_capture_system：支持 Neutral 城池被攻击至 HP ≤ 0 时翻转为攻击方阵营

## 4. 输入与选择系统（新模块 src/input/mod.rs）

- [ ] 4.1 新建 src/input/mod.rs，定义 InputPlugin、SelectionState Resource、SelectionMode 枚举、SelectionIndicator Component
- [ ] 4.2 实现 selection_click_system：点击己方士兵选中（附加 SelectionIndicator）、点击城池/空地按规则处理（桌面端左键 + 移动端单指）
- [ ] 4.3 实现 drag_select_system：左键/单指拖拽框选（矩形/圆形模式），拖拽起点命中己方单位时进入框选（不触发摄像机平移），拖拽起点空地时摄像机平移（移动端）
- [ ] 4.4 实现 command_issue_system：右键（桌面）/轻点空地/敌方（移动端）下达移动/攻击指令。敌方 Entity → target 为敌方；空地 → 生成 Waypoint Entity
- [ ] 4.5 实现 selection_visual_system：选中士兵附加/更新半透明绿色圆环 SelectionIndicator
- [ ] 4.6 实现快捷键系统：Ctrl+A 全选己方士兵，Esc 取消所有选中（与暂停功能的 Esc 协调——如果无选中则暂停，有选中则取消选中）
- [ ] 4.7 框选时渲染拖拽选区视觉（矩形半透明框或圆形半透明框，用 Lyon Shape 跟随鼠标/手指）
- [ ] 4.8 在 main.rs 注册 InputPlugin，确保系统在 CameraSystem 之后、UISystem 之前运行

## 5. UI 系统修复 (UI Fixes)

- [ ] 5.1 所有 UI 文件全局替换 emoji 为纯文本：menu.rs（标题+按钮文本）、hud.rs（状态栏+面板+工具栏）、pause.rs（暂停菜单）
- [ ] 5.2 新增 update_top_bar_system：每帧更新顶部状态栏 Text（玩家城池数/总城池数、总人口、运行时间 mm:ss）
- [ ] 5.3 实现暂停按钮 ⏸️ 的点击逻辑：检测 Interaction::Pressed → next_state.set(Paused)
- [ ] 5.4 实现 update_bottom_panel：监听 CitySelectedEvent，读取城池数据，更新面板文本（等级/上限、人口）并设为 Display::Flex
- [ ] 5.5 底部面板血量用 Bevy Node 实现（灰色背景 Node + 绿色/红色填充子 Node，宽度 = HP 百分比）
- [ ] 5.6 实现 soldier_type_button_system：4 个兵种按钮点击 → 更新选中城池 spawn_type，高亮当前选中兵种按钮
- [ ] 5.7 实现 shield_toggle_system：工具栏举盾按钮切换所有选中步兵的 InfantryShield 状态（混合→全部举盾→全部正常）
- [ ] 5.8 实现圆形/方形框选模式切换按钮功能

## 6. AI 完整行为 (AI Behavior)

- [ ] 6.1 完善 AI 防御评估：城池 HP < 50% 时，将以该城为 origin 且在光环范围内的 50% 士兵 target 设为该城（回防）
- [ ] 6.2 新增 AI 扩张评估：找到最近中立城池，500px 范围内 AI 兵力 > 中立城 MaxHealth × 1.5 时，派兵进攻该中立城
- [ ] 6.3 新增 AI 进攻评估：找到最近敌方城池（等级 ≤ 己方最高等级），周边 AI 兵力 > 敌方兵力 × 1.3 时，派兵进攻
- [ ] 6.4 新增 AI 升级评估：己方城池 population > MaxPopulation × 0.6 时，将多余士兵 target 设回该城
- [ ] 6.5 实现 count_nearby_soldiers 辅助函数：计算某位置半径范围内指定 faction 的士兵数和总攻击力
- [ ] 6.6 AI 决策系统添加对 Soldier Query 的写权限，通过设置 soldier.target 向士兵下达指令

## 7. GameOver 结算面板 (GameOver Panel)

- [ ] 7.1 新建 src/ui/gameover.rs，定义 GameOverPlugin（OnEnter 设 UI、OnExit 清理、Update 按钮系统）
- [ ] 7.2 新增 GameStats Resource（start_time, total_kills），在 OnEnter(Playing) 初始化，在 SoldierDiedEvent observer 累加击杀
- [ ] 7.3 结算面板 UI：显示胜负结果、游戏时间（mm:ss）、剩余城池数、总击杀数（纯文本，无 emoji）
- [ ] 7.4 实现 "再来一局" 按钮：切换到 Playing 状态（触发 OnEnter Playing 清理+重新生成）
- [ ] 7.5 实现 "返回主菜单" 按钮：切换到 MainMenu 状态
- [ ] 7.6 在 ui/mod.rs 和 main.rs 中注册 GameOverPlugin

## 8. 编译与验证

- [ ] 8.1 每一层完成后 cargo check 验证编译通过
- [ ] 8.2 全流程 cargo check 确保无编译错误
- [ ] 8.3 全流程 cargo run 验证游戏可启动、单人模式可正常操作
- [ ] 8.4 验证：玩家可选兵+框选+指挥移动/攻击
- [ ] 8.5 验证：底部面板/顶部状态栏正常刷新数据
- [ ] 8.6 验证：AI 主动进攻/扩张/防守/升级
- [ ] 8.7 验证：GameOver 结算面板正确显示
