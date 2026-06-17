## Context

当前 `city_interaction_system` 中，当士兵到达己方城池（`is_targeted == true`），无论城池是否需要治愈或升级贡献，士兵都会被无条件 despawn 并扣减出生城人口（`simulation/src/soldier/mod.rs:554-555`）。

同时，`combat_target_system` 中，当处于 Fighting 状态的士兵的攻击目标消失后，`command_target` 被错误地清空为 `None`（`simulation/src/combat/mod.rs:131`），导致玩家原始移动/回城意图丢失。

## Goals / Non-Goals

**Goals:**
- 当城池满血 + 满级时，到达的己方士兵不被 despawn，停在原位
- 当攻击目标消失时，保留 `command_target`，士兵恢复向原目标移动

**Non-Goals:**
- 不改变城池的治愈/升级机制
- 不改变士兵移动系统的到达判定逻辑
- 不改变 overlap_resolution 的碰撞规则

## Decisions

### Decision 1：Bug 1 修复 — 条件守卫 despawn

将 `soldier/mod.rs:554-555` 的 despawn 逻辑移入 `if ci.hp < max_hp` 和 `else if ci.level < max_level` 分支内。当两个条件都不满足时，不执行 despawn，士兵保持在当前位置。

```rust
// 修改前 (530-556):
if is_targeted {
    if ci.hp < ci.max_hp {
        // heal
    } else if ci.level < ci.max_level {
        // level up
    }
    // ← 无条件 despawn (BUG)
    if let Some(o) = ... { origin_decrements.push(o.0); }
    to_despawn.push((si.entity, None));
    break;
}

// 修改后:
if is_targeted {
    let mut consumed = false;
    if ci.hp < ci.max_hp {
        // heal
        consumed = true;
    } else if ci.level < ci.max_level {
        // level up
        consumed = true;
    }
    if consumed {
        if let Some(o) = ... { origin_decrements.push(o.0); }
        to_despawn.push((si.entity, None));
    }
    break;
}
```

### Decision 2：Bug 2 修复 — 保留 command_target

将 `combat/mod.rs:131` 的 `command_target: None` 改为 `command_target: sd.cmd_target`。

```rust
// 修改前:
em.insert(Movement { speed: sd.speed, target: ct, command_target: None, waypoint: None, force_move: false });

// 修改后:
em.insert(Movement { speed: sd.speed, target: ct, command_target: sd.cmd_target, waypoint: None, force_move: false });
```

## Risks / Trade-offs

- **[风险] 士兵滞留在城池位置**：满级满血城池旁的士兵会累积停在原位，可能造成视觉拥挤。→ **缓解**：当前 overlap_resolution 系统会处理同位置碰撞；后续可考虑给满级城池加"巡逻/驻守"行为，但属于 Non-Goal。
- **[风险] command_target 保留后士兵行为变化**：保留 command_target 意味着士兵在战斗结束后会继续向原目标移动，这是预期行为。→ **缓解**：无，这正是期望的。
