## Context

simulation-multiplayer-slots 已添加 multi_player() 和 init_simulation_world_multi()。网络模式现在需要传入 player_count 以使用动态 PlayerSlots。

## Goals

NetworkGameStart 新增 player_count → setup_lobby_system 写入 → reset_game_system 使用 init_simulation_world_multi

## Decisions

3 处数据流变更：
1. struct NetworkGameStart { player_count: u8 }
2. setup_lobby_system: network_start.player_count = player_count
3. reset_game_system: init_simulation_world_multi(seed, PlayerSlots::multi_player(...))

## Risks

单机模式不受影响（Network 路径只在网络模式下走）
