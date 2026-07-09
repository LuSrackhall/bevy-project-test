## Why

当前 PlayerSlots 仅支持 2 玩家。需扩展支持 3-4 玩家多人 FFA。

## What Changes

- PlayerSlots::multi_player() 类构造函数
- init_simulation_world_multi() 重载

## Impact

- crates/simulation/src/types.rs — +multi_player()
- crates/simulation/src/lib.rs — +init_simulation_world_multi()
