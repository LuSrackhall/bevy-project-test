## Context

当前 `PlayerSlots::single_player()` 硬编码 2 个槽位（Human+AI）。需支持 N 个玩家。

## Goals

- `PlayerSlots::multi_player(count, local_player_id)` 生成 N 个人类槽位
- `init_simulation_world_multi(seed, slots)` 新函数（保持 `init_simulation_world` 向后兼容）
- `test_multi_player_slots_3` / `test_multi_player_slots_4` 测试

## Decisions

### D1: multi_player(count, local_player_id)

```rust
pub fn multi_player(count: u8, local_player_id: u8) -> Self {
    assert!(count <= 8, "max 8 players");
    let slots = (0..count).map(|i| PlayerSlot {
        slot_id: SlotId(i),
        controller: if i == local_player_id {
            Controller::HumanLocal
        } else {
            Controller::HumanRemote(SlotId(i))
        },
        faction: FactionId(i),
        team: TeamId(0),
    }).collect();
    Self { slots }
}
```

### D2: 向后兼容

`init_simulation_world(seed)` → 默认 `single_player()`  
`init_simulation_world_multi(seed, slots)` → 新重载

## Risks

单机兼容: `init_simulation_world(seed)` 行为不变
