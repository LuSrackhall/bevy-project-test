# 城池争霸 — RTS 游戏设计规格

## 概述

基于 Bevy 引擎的极简 RTS 游戏。圆圈代表城池，城池自动产出士兵，玩家通过框选指挥士兵攻占敌方城池。支持单人对战 AI，优先完成后可扩展为多人联机。

## 技术选型

- **引擎**：Bevy 0.15
- **2D 绘制**：bevy_prototype_lyon 0.13（城池圆环、血条、框选）
- **随机数**：rand 0.8
- **架构**：纯 ECS，每个模块独立 Plugin，通过 Event + Resource 通信

## 模块划分

```
src/
├── main.rs              # App 入口，注册所有插件
├── core/                # 核心常量、兵种定义
├── camera/              # 摄像机插件（拖拽平移 + 缩放）
├── map/                 # 地图生成插件
├── city/                # 城池插件（Component、血量、等级、归属、产兵）
├── soldier/             # 士兵插件（Component、移动、战斗状态）
├── ai/                  # 人机 AI 插件
├── ui/                  # UI 插件（面板、框选、菜单）
├── combat/              # 战斗系统（弓箭、伤害、闪避、减速）
└── game/                # 游戏流程（状态机、胜利判定）
```

---

## 一、兵种系统

### 属性表

| 属性 | 民兵 | 步兵 | 弓兵 | 骑兵 |
|------|------|------|------|------|
| 生命值 | 100 | 100 | 100 | 140 |
| 攻击力 | 16 | 20 | 20 | 20 |
| 移动速度 | 80 px/s | 80 px/s | 80 px/s | 200 px/s |
| 攻击范围 | 近战(30px) | 近战(30px) | 远程(600px) | 近战(30px) |
| 攻击间隔 | 0.5s | 0.5s | 0.5s | 0.5s |
| 产出速度倍率 | 1.0x | 0.5x | 0.5x | 0.3x |

### 特殊效果

- **民兵**：无特殊效果
- **步兵**：受到弓兵伤害 -10%（即弓兵对步兵伤害 ×0.9）
- **弓兵**：
  - 远程攻击（600px），生成 Arrow Entity 飞向目标（飞行速度 400px/s，追踪目标）
  - 命中附带减速效果：首次命中移速 ×0.85（减速 15%），持续 1s；持续时间内再次命中叠加 ×0.9（额外减速 10%），刷新 1s 持续时间
  - 减速叠加公式：`移速 × 0.85 × 0.9^(n-1)`，上限为移速降至原始速度的 35%
  - 目标 ≤ 50px 时伤害 ×0.85（近程惩罚）
- **骑兵**：
  - 闪避机制：`闪避率 = 50% - (1 - 当前血量/最大血量) × 35%`
  - 满血 50%、半血 32.5%、10%血 18.5%
  - 血量 < 10% 时闪避完全失效（0%）

### 伤害计算

```
基础伤害 = 攻击者攻击力
  × 弓兵对步兵: 0.9（步兵免弓伤）
  × 弓兵近程惩罚: 目标 ≤ 50px → ×0.85
  × 其余组合: 1.0
  [骑兵目标] → roll 闪避 → 成功则伤害 = 0
```

### 兵种切换

- 所有城池默认可生产全部 4 种兵种，无需解锁
- 用户点击城池 → 底部面板显示 4 个兵种按钮 → 点击切换
- 切换后仅影响后续产出，已存在的士兵保持原兵种
- 每个士兵占用城池人口上限的 1 个位置，不论兵种

---

## 二、城池系统

### 属性

- CityLevel: 当前等级
- CityMaxLevel: 等级上限（基础 rand(3..7)，浮动 clamp(基础+rand(-1..+2), 1, 10)）
- CityHealth / CityMaxHealth: 血量（MaxHealth = Level × 100）
- CityPopulation / CityMaxPopulation: 人口（MaxPopulation = Level × 12 + rand(Level×2 .. Level×5)）
- CityFaction: Player | Enemy | Neutral
- CitySpawnType: 当前出兵兵种
- CitySpawnTimer: 产兵计时器
- CityPosition: 地图坐标

### 士兵进城（全部即时消耗士兵）

| 场景 | 效果 |
|------|------|
| 进入敌方/中立城池 | 士兵消失，城池扣血 = 士兵攻击力 × 0.5 |
| 进入己方残血城池 | 士兵消失，回复 MaxHealth × 30% |
| 进入己方满血 + 等级未满 | 士兵消失，增加 MaxHealth × 30% 升级经验。升级经验阈值为 MaxHealth × 100，达到即升级 |

### 城池攻克

- CityHealth ≤ 0 → Faction 翻转为攻击方
- Level = max(1, Level - 1)（1级城保持1级）
- Health = 新等级 MaxHealth × 20%
- 被攻陷方由该城产出的士兵不死亡，且不占用其他城池名额上限。这些士兵自动变为"流亡士兵"，继续存在于地图上，归属不变，可继续战斗，但没有产出城池（无法补充）

### 阵营分配（对称）

- 中立城池数量 = rand(N × 0.3 .. N × 0.5)
- 玩家城池 = ceil((N - 中立) / 2)
- AI 城池 = ceil((N - 中立) / 2)
- 确保玩家和 AI 至少各 1 座

---

## 三、地图生成

### 算法

1. 城池总数 N = rand(6..20)
2. 对每座城池：随机坐标在 [margin, map_size - margin] 内
3. 与已放置城池距离 ≥ city_min_distance（不满足则重试最多 100 次）
4. 分配 Level、MaxLevel、Faction
5. 地图默认尺寸 2000×2000 px

---

## 四、战斗系统

### 战斗流程

```
士兵进入攻击范围 → SoldierState = Fighting
  → 每 0.5s 攻击间隔判定：
      [骑兵目标] → roll 闪避 → 成功 → 无伤害
      [其余目标] → 计算伤害 → 扣除血量
      [弓兵] → 生成 Arrow Entity（追踪目标，400px/s）
             → 到达后结算伤害 + 减速 debuff
  → 目标血量 ≤ 0 → 目标 Entity 销毁
```

### 弓兵特殊处理

- 唯一的远程弹道兵种
- 在 600px 外锁敌开始攻击，生成 Arrow Entity
- Arrow 追踪目标（即使目标移动），到达时结算伤害
- 目标 ≤ 50px 时伤害 ×0.85
- 减速可叠加，最高减速至原始速度的 35%

---

## 五、AI 行为

AI 每 2 秒评估一次全局状态，按优先级执行：

1. **防御评估**：己方城池血量 < 50% 则切换克制兵种，邻近城池派 50% 产出支援
2. **扩张评估**：攻击最近中立低等级城池（兵力 > 目标 MaxHealth × 1.5）
3. **进攻评估**：攻击最近敌方 ≤ 己方最高等级的城池（周边兵力 > 敌方 × 1.3）
4. **升级评估**：多余兵力（人口 > 上限 × 0.6）回城升级

周边兵力评估范围：目标城池半径 500px

---

## 六、UI 设计（移动端 + 桌面端双端适配）

### 屏幕布局

```
┌──────────────────────────┐
│  🏰4/3  👥87   ⏱️3:24 ⏸️│  ← 顶部状态栏
│                          │
│        [游戏画面]         │
│                          │
├──────────────────────────┤
│  🏰 城池 Lv.3           │  ← 底部面板（选中城池时出现）
│  ❤️ ████████░░ 240/300  │
│  👥 18/45  ⭐上限:5     │
│  ─────────────────────  │
│  [民兵][步兵][弓兵][骑兵] │
├──────────────────────────┤
│  [⭕框选] [⬜框选]        │  ← 底部工具栏
└──────────────────────────┘
```

### 双端操作对照

| 操作 | 移动端 | 桌面端 |
|------|--------|--------|
| 摄像机平移 | 单指拖拽 | 鼠标拖拽（中键/右键） |
| 摄像机缩放 | 双指捏合 | 鼠标滚轮 |
| 点击城池 | 单指点击 | 鼠标左键 |
| 框选士兵 | 拖拽（圆形/方形） | 鼠标左键拖拽框选 |
| 移动/攻击 | 点击目标位置 | 右键点击目标位置 |
| 兵种切换 | 底部面板点击 | 底部面板点击 |

### 菜单层级

- **主菜单**：单人模式 / 多人模式(灰) / 设置 / 帮助
- **暂停菜单**（点击 ⏸️）：继续 / 设置 / 重新开始 / 返回主菜单
- **设置菜单**：音乐音量 / 音效音量 / 框选模式(圆形/方形) / 语言
- **结算面板**（GameOver）：胜利/失败、游戏时间、剩余城池、总击杀、再来一局 / 返回主菜单

---

## 七、游戏流程

### 状态机

```
MainMenu → Playing → Paused → Playing
                 ↘ GameOver → MainMenu
```

### 状态行为

| 状态 | 渲染 | 逻辑更新 | 输入 |
|------|------|---------|------|
| MainMenu | 主菜单 UI | 无 | 菜单交互 |
| Playing | 地图 + UI | 全部运行 | 游戏操作 |
| Paused | 地图(模糊) + 暂停菜单 | 冻结 | 菜单交互 |
| GameOver | 地图(灰色) + 结算面板 | 冻结 | 菜单交互 |

### 胜利条件

- 消灭敌方所有城池（攻占所有敌方城池即获胜）

### 启动流程

1. App 启动 → 注册所有 Plugin
2. 进入 MainMenu → 玩家点击"单人模式"
3. 生成随机地图 → 分配城池 → 摄像机定位到玩家城池
4. 进入 Playing

---

## 八、核心数据模型

### Component

- **City**（标记组件）: CityLevel, CityMaxLevel, CityHealth, CityMaxHealth, CityFaction, CityPopulation, CityMaxPopulation, CitySpawnTimer, CitySpawnType, CityPosition
- **Soldier**（标记组件）: SoldierType, SoldierFaction, SoldierHealth, SoldierMaxHealth, SoldierSpeed, SoldierTarget, SoldierState
- **Arrow**（标记组件）: ArrowTarget, ArrowDamage, ArrowSpeed, ArrowSlow

### Resource

- **GameConfig**: map_width, map_height, min_cities, max_cities, city_min_distance, faction
- **GameState**: MainMenu | Playing | Paused | GameOver(Winner)

### Event

- **CitySelected** { entity, faction } — UI → City
- **CityCaptured** { entity, new_faction } — City → Game
- **SoldierSpawned** { entity, city_entity } — City → Soldier
- **SoldierDied** { entity, killer_entity } — Combat → City（击杀计数）

---

## 九、Bevy 系统调度顺序

```
Update 阶段：
1. InputSystem        # 处理触摸/鼠标输入
2. CameraSystem       # 摄像机拖拽缩放
3. SelectionSystem    # 框选逻辑
4. CitySystem         # 城池血量、升级、产兵
5. SoldierSystem      # 士兵移动、状态机
6. CombatSystem       # 战斗判定（弓箭、伤害、闪避、减速）
7. AISystem           # AI 决策
8. UISystem           # UI 面板更新
9. GameStateSystem    # 胜利条件检测
```
