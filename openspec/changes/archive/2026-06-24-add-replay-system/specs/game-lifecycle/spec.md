## MODIFIED Requirements

### Requirement: GameState 三变体枚举

GameState SHALL 保持现有三个变体（MainMenu、Playing、GameOver）。Replay 模式 SHALL NOT 作为 GameState 变体，而是通过 bevy_adapter 的 GameMode 枚举控制。进入 Replay 模式时 GameState 保持为 Playing。

#### Scenario: Replay 模式下的 GameState
- **WHEN** 用户从主菜单加载 Replay 文件
- **THEN** GameState 切换为 Playing，同时 GameMode 切换为 Replay

## ADDED Requirements

### Requirement: 主菜单 Replay 按钮

render_view 的主菜单 SHALL 包含 "Load Replay" 按钮，点击后打开文件选择器，选择 .replay 文件后进入 Replay 模式。

#### Scenario: 加载 Replay 文件
- **WHEN** 用户在主菜单点击 "Load Replay"
- **AND** 选择一个有效的 .replay 文件
- **THEN** GameState 切换为 Playing，GameMode 切换为 Replay，从 tick 0 开始播放

#### Scenario: 加载无效文件
- **WHEN** 用户选择一个格式错误或版本不匹配的 .replay 文件
- **THEN** 显示错误信息，保持在主菜单

### Requirement: Replay 播放器 UI

render_view SHALL 在 Replay 模式下显示播放器控制栏：播放/暂停按钮、快进选择（1x/2x/4x）、进度条（可拖拽 seek）、tick 计数器。

#### Scenario: 进度条拖拽 seek
- **WHEN** 用户拖拽进度条到 50% 位置
- **AND** 总 tick 为 12000
- **THEN** 设置 target_tick = 6000，系统从当前 tick 快放到 6000

#### Scenario: 快进切换
- **WHEN** 用户点击 4x 按钮
- **THEN** 播放速度从 1x 切换为 4x

### Requirement: 录制开关设置

render_view SHALL 在设置界面提供 "自动录制 Replay" 开关，默认开启。关闭后 GameOver 时 ReplayRecorder 不写入文件。

#### Scenario: 默认开启录制
- **WHEN** 首次运行游戏
- **THEN** 自动录制 Replay 默认为开启状态

#### Scenario: 关闭录制后不保存
- **WHEN** 用户关闭自动录制
- **AND** 游戏结束
- **THEN** 不生成 .replay 文件

### Requirement: GameOver 时 Replay 保存

当 GameState 从 Playing 切换到 GameOver 且录制已启用时，render_view SHALL 触发 ReplayRecorder 将录制数据写入磁盘。

#### Scenario: 自动保存
- **WHEN** 游戏结束且录制已启用
- **THEN** 生成 .replay 文件，文件名包含时间戳
