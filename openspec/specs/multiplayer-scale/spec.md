# multiplayer-scale Specification

## Purpose
玩家数量参数化:创建房间人数 2..=8、current_players 实时更新。
## Requirements
## ADDED Requirements

### Requirement: Room capacity parameterization

创建房间的玩家容量 SHALL 是可配置参数,不再硬编码为 2。UI 创建房间时 SHALL 提供 2..=8 的人数选择。

#### Scenario: room created with 4 players
- **WHEN** 玩家创建房间并选择人数为 4
- **THEN** 房间的 `max_players` 为 4,relay 以 4 人容量启动,`JoinGame` 满员判定按 4 执行

#### Scenario: room created with default capacity
- **WHEN** 玩家创建房间且未显式选择人数
- **THEN** `max_players` 使用默认值 2(保留现有默认行为)

#### Scenario: capacity selector cycles within range
- **WHEN** 玩家点击创建房间的人数选择器
- **THEN** 人数在 2..=8 范围内循环切换,不会超出该范围

### Requirement: current_players accurate reporting

房间的 `current_players` SHALL 随玩家加入/离开实时更新,不再恒为 1。满员判定 SHALL 基于真实的 `current_players >= max_players`。

#### Scenario: second player joins room
- **WHEN** 一个玩家加入已有 1 名玩家的房间
- **THEN** 房间 `current_players` 变为 2

#### Scenario: player leaves room
- **WHEN** 房间内一名玩家离开(掉线或退出)
- **THEN** 房间 `current_players` 减 1

#### Scenario: full room rejects join
- **WHEN** `current_players >= max_players` 时新玩家尝试加入
- **THEN** 加入请求被拒绝,房间列表不显示"可加入"
