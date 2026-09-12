use crate::command::*;
use crate::soldier::config::SoldierConfig;
use crate::soldier::*;
use crate::types::*;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;

fn rng_range(rng: &mut DeterministicRng, min: u32, max: u32) -> u32 {
    let range = max.wrapping_sub(min).wrapping_add(1);
    if range == 0 {
        return min;
    }
    (rng.next_u64() as u32) % range + min
}

const AI_TICK_INTERVAL: u32 = 40;

/// 发起一次远征/进攻所需的最小可派遣兵力。
/// 门槛的意义是"不要派一两个士兵去送死"，而不是"等士兵自己走到目标附近"。
const AI_MIN_EXPEDITION: usize = 3;

/// AI 决策入口。读取 PlayerSlots，为每个 AI Controller 的 faction 生成命令。
///
/// Complexity: O(c * s)  [c=己方城池数, s=士兵数]
/// Memory:     O(s)      [每 40 tick 构建一次快照]
/// Hot Path:   No         [AI_TICK_INTERVAL=40，非每 Tick 路径]
pub fn ai_decide(world: &mut World, current_tick: u32) {
    if !current_tick.is_multiple_of(AI_TICK_INTERVAL) {
        return;
    }

    let slots = world.get_resource::<PlayerSlots>();
    let ai_slots: Vec<FactionId> = match slots {
        Some(s) => s
            .slots
            .iter()
            .filter(|s| matches!(s.controller, Controller::AI(_)))
            .map(|s| s.faction)
            .collect(),
        None => vec![FactionId(1)], // fallback: legacy Enemy
    };

    let _soldier_config = world.resource::<SoldierConfig>().clone();

    for &ai_faction in &ai_slots {
        ai_decide_for_faction(world, current_tick, ai_faction);
    }
}

/// AI 为单个 faction 生成命令（从原 ai_decide 提取）。
fn ai_decide_for_faction(world: &mut World, current_tick: u32, ai_faction: FactionId) {
    // Collect AI (Enemy) cities — sorted for determinism (§0.1)
    let mut ai_cities: Vec<(UnitId, FixedVec2, u32, u32)> = {
        let mut query = world.query::<(
            Entity,
            &UnitIdComponent,
            &LogicalPosition,
            &CityComponent,
            &FactionComponent,
        )>();
        query
            .iter(world)
            .filter(|(_, _, _, _, fac)| fac.0 == ai_faction)
            .map(|(_, id, pos, city, _)| (id.0, pos.0, city.level, city.max_level))
            .collect()
    };
    ai_cities.sort_by_key(|(uid, _, _, _)| *uid);

    // Collect enemy cities — 所有非己方、非中立阵营，按 UnitId 排序保证确定性（§0.1）。
    // 原实现硬编码 FactionId(0)：既使 AI 无法与另一个 AI 对抗（对称自对弈不可用），
    // 也无法支持 >2 个非中立阵营。
    let mut enemy_cities: Vec<(UnitId, FixedVec2, u32)> = {
        let mut query = world.query::<(
            Entity,
            &UnitIdComponent,
            &LogicalPosition,
            &CityComponent,
            &FactionComponent,
        )>();
        query
            .iter(world)
            .filter(|(_, _, _, _, fac)| fac.0 != ai_faction && fac.0 != FactionId(2))
            .map(|(_, id, pos, city, _)| (id.0, pos.0, city.level))
            .collect()
    };
    enemy_cities.sort_by_key(|(uid, _, _)| *uid);

    // Collect Neutral cities — sorted for determinism (§0.1)
    let mut neutral_cities: Vec<(UnitId, FixedVec2, u32, u32)> = {
        let mut query = world.query::<(
            Entity,
            &UnitIdComponent,
            &LogicalPosition,
            &CityComponent,
            &FactionComponent,
        )>();
        query
            .iter(world)
            .filter(|(_, _, _, _, fac)| fac.0 == FactionId(2))
            .map(|(_, id, pos, city, _)| (id.0, pos.0, city.level, city.health_max))
            .collect()
    };
    neutral_cities.sort_by_key(|(uid, _, _, _)| *uid);

    // Collect all soldiers — sorted for determinism (§0.1)
    let mut soldiers: Vec<(UnitId, FixedVec2, FactionId, bool, Option<UnitId>)> = {
        let mut query = world.query::<(
            Entity,
            &UnitIdComponent,
            &LogicalPosition,
            &FactionComponent,
            &Movement,
        )>();
        query
            .iter(world)
            .map(|(_, id, pos, fac, mov)| {
                // 已派遣 = 有攻击目标 或 有行军路点。
                // MoveTo 设置的是 waypoint（见 soldier::apply_movement），只看 target 会把
                // 正在行军的士兵误判为"空闲"，导致每 40 tick 重复下发同一行军命令。
                (
                    id.0,
                    pos.0,
                    fac.0,
                    mov.target.is_some() || mov.waypoint.is_some(),
                    mov.command_target,
                )
            })
            .collect()
    };
    soldiers.sort_by_key(|(uid, _, _, _, _)| *uid);

    // Expansion + Attack + Upgrade
    let mut commands: Vec<GameCommand> = Vec::new();

    for &(_ai_city_id, ai_pos, ai_level, _ai_max_level) in &ai_cities {
        // Expansion: target nearest neutral city
        if !neutral_cities.is_empty() {
            let mut by_dist: Vec<(usize, i64)> = neutral_cities
                .iter()
                .enumerate()
                .map(|(i, (_uid, npos, _, _))| (i, (ai_pos - *npos).length_squared().0))
                .collect();
            by_dist.sort_by_key(|(i, d)| (*d, neutral_cities[*i].0));

            let (idx, _) = by_dist[0];
            let (_target_city_id, target_pos, _, _target_hp) = neutral_cities[idx];
            let radius_sq = Fixed::from_int(500) * Fixed::from_int(500);

            // 可派遣兵力 = 位于本城附近且尚未派遣的己方士兵（soldiers 已按 UnitId 排序）。
            // 关键修正：原实现用"目标城附近已有己方士兵"（ai_nearby > 0）作为门槛，
            // 而士兵初始都待在本城 → 第一波永远派不出去（死锁，AI 事实上不存在）。
            // 现改为"本城可派遣兵力达到 AI_MIN_EXPEDITION"。
            let dispatchable: Vec<UnitId> = soldiers
                .iter()
                .filter(|(_, spos, sfac, is_committed, _)| {
                    *sfac == ai_faction
                        && !*is_committed
                        && (*spos - ai_pos).length_squared() <= radius_sq
                })
                .map(|(sid, _, _, _, _)| *sid)
                .collect();

            if dispatchable.len() >= AI_MIN_EXPEDITION {
                for sid in dispatchable {
                    commands.push(GameCommand {
                        tick: current_tick + 1,
                        player_id: ai_faction.0,
                        action: Action::MoveTo {
                            unit: sid,
                            target: target_pos,
                        },
                    });
                }
            }
        }

        // Attack: 进攻最近的敌方城市（敌方 = 所有非己方、非中立阵营）
        if !enemy_cities.is_empty() {
            let mut by_dist: Vec<(usize, i64)> = enemy_cities
                .iter()
                .enumerate()
                .map(|(i, (_uid, ppos, _))| (i, (ai_pos - *ppos).length_squared().0))
                .collect();
            by_dist.sort_by_key(|(i, d)| (*d, enemy_cities[*i].0));

            for &(idx, _) in &by_dist {
                let (_target_city_id, target_pos, enemy_level) = enemy_cities[idx];
                if ai_level >= enemy_level {
                    let radius_sq = Fixed::from_int(500) * Fixed::from_int(500);
                    let ai_nearby = soldiers
                        .iter()
                        .filter(|(_, pos, fac, _, _)| {
                            *fac == ai_faction && (*pos - target_pos).length_squared() <= radius_sq
                        })
                        .count();
                    let enemy_nearby = soldiers
                        .iter()
                        .filter(|(_, pos, fac, _, _)| {
                            *fac != ai_faction
                                && *fac != FactionId(2)
                                && (*pos - target_pos).length_squared() <= radius_sq
                        })
                        .count();
                    // 可用兵力 = 已在目标附近 + 可从本城派出（同样修正死锁门槛：
                    // 只看"已在附近"会让第一波永远派不出去）
                    let dispatchable = soldiers
                        .iter()
                        .filter(|(_, spos, sfac, is_committed, _)| {
                            *sfac == ai_faction
                                && !*is_committed
                                && (*spos - ai_pos).length_squared() <= radius_sq
                        })
                        .count();
                    let available = ai_nearby + dispatchable;

                    if (available as u64) * 10 > (enemy_nearby as u64) * 13 && available > 0 {
                        for &(sid, _spos, sfac, is_committed, _) in &soldiers {
                            if sfac == ai_faction && !is_committed {
                                commands.push(GameCommand {
                                    tick: current_tick + 1,
                                    player_id: ai_faction.0,
                                    action: Action::MoveTo {
                                        unit: sid,
                                        target: target_pos,
                                    },
                                });
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    // Defense: low HP cities switch spawn and recall
    {
        let mut low_hp_cities: Vec<UnitId> = {
            let mut query =
                world.query::<(Entity, &UnitIdComponent, &CityComponent, &FactionComponent)>();
            query
                .iter(world)
                .filter(|(_, _, city, fac)| {
                    fac.0 == ai_faction && city.health_current < city.health_max / 2
                })
                .map(|(_, id, _, _)| id.0)
                .collect()
        };
        low_hp_cities.sort();
        for city_id in low_hp_cities {
            let rng_val = {
                let mut rng = world.resource_mut::<DeterministicRng>();
                rng_range(&mut rng, 0, 3)
            };
            let st = match rng_val {
                0 => SoldierType::Infantry,
                1 => SoldierType::Archer,
                _ => SoldierType::Cavalry,
            };
            commands.push(GameCommand {
                tick: current_tick + 1,
                player_id: ai_faction.0,
                action: Action::SetSpawnType {
                    city: city_id,
                    soldier_type: st,
                },
            });
        }
    }

    // Push all commands
    let mut cmd_buf = world.resource_mut::<CommandBuffer>();
    for cmd in commands {
        cmd_buf.push(cmd);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{self, MapSize};
    use crate::world_stats::{count_factions, FactionCounts};
    use crate::{init_simulation_world_multi, run_tick, RunConfig};

    /// 一局无头对局的可观测结果。
    struct MatchOutcome {
        spawned: u64,
        destroyed: u64,
        captured: u64,
        factions: FactionCounts,
    }

    fn run_match(seed: u64, ticks: u32, slots: PlayerSlots) -> MatchOutcome {
        let mut world = init_simulation_world_multi(seed, slots);
        map::generate_map(&mut world, MapSize::Small);
        let config = RunConfig { enable_ai: true };

        let (mut spawned, mut destroyed, mut captured) = (0u64, 0u64, 0u64);
        for tick in 1..=ticks {
            let ev = run_tick(&mut world, tick, &config);
            spawned += ev.spawned.len() as u64;
            destroyed += ev.destroyed.len() as u64;
            captured += ev.captured.len() as u64;
        }

        MatchOutcome {
            spawned,
            destroyed,
            captured,
            factions: count_factions(&mut world),
        }
    }

    /// 全部槽位由 AI 控制（对称自对弈）。
    fn all_ai_slots(players: u8) -> PlayerSlots {
        let slots = (0..players)
            .map(|i| PlayerSlot {
                slot_id: SlotId(i),
                controller: Controller::AI(AiProfile::default()),
                faction: FactionId(i),
                team: TeamId(i),
            })
            .collect();
        PlayerSlots { slots }
    }

    /// faction 0 被动、其余为 AI。
    fn ai_vs_passive_slots(players: u8) -> PlayerSlots {
        let slots = (0..players)
            .map(|i| PlayerSlot {
                slot_id: SlotId(i),
                controller: if i == 0 {
                    Controller::HumanLocal
                } else {
                    Controller::AI(AiProfile::default())
                },
                faction: FactionId(i),
                team: TeamId(i),
            })
            .collect();
        PlayerSlots { slots }
    }

    /// 非中立且仍持有城池的阵营。
    fn contested_factions(counts: &FactionCounts) -> Vec<u8> {
        counts
            .factions
            .iter()
            .filter(|(f, (_, cities))| f.0 != FactionId(2).0 && *cities > 0)
            .map(|(f, _)| f.0)
            .collect()
    }

    /// 回归防线：AI 必须真的发起进攻。
    ///
    /// 历史缺陷（2026-09-12 修复）：扩张/进攻的门槛是"**目标城附近已有己方士兵**"
    /// （`ai_nearby > 0`），而士兵初始都待在本城 → 第一波永远派不出去。
    /// 表现：4000 与 12000 tick 下 `spawned` 恒为 60、`destroyed/captured` 恒为 0，
    /// 世界在初始产兵后彻底冻结，AI 对手事实上不存在。
    #[test]
    fn ai_prosecutes_offensive_within_budget() {
        let outcome = run_match(42, 4000, ai_vs_passive_slots(2));
        assert!(
            outcome.destroyed > 0,
            "AI 必须在预算内造成击杀，实际 spawned={} destroyed={} captured={} factions={:?}",
            outcome.spawned,
            outcome.destroyed,
            outcome.captured,
            outcome.factions.factions
        );
        assert!(
            outcome.captured > 0,
            "AI 必须攻占城池，实际 captured={}",
            outcome.captured
        );
    }

    /// P0「机器验收」的 AI 自对弈门：对称自对弈必须在预算内决出唯一胜者。
    ///
    /// 同时验证敌方选择的泛化——原实现把对手硬编码为 `FactionId(0)`，
    /// 使得两个 AI 无法互相对抗（faction 0 的 AI 只会打自己）。
    #[test]
    fn symmetric_selfplay_reaches_decision() {
        let outcome = run_match(42, 4000, all_ai_slots(2));
        let contested = contested_factions(&outcome.factions);
        assert_eq!(
            contested.len(),
            1,
            "对称自对弈应在预算内决出唯一胜者，实际城池分布 {:?}（destroyed={} captured={}）",
            outcome.factions.factions,
            outcome.destroyed,
            outcome.captured
        );
        assert!(
            outcome.destroyed > 0,
            "对称自对弈必须有实际交战，实际 destroyed={}",
            outcome.destroyed
        );
    }

    /// 确定性：同种子的自对弈结果必须逐位一致（§10.1）。
    #[test]
    fn selfplay_is_deterministic() {
        let a = run_match(7, 1500, all_ai_slots(2));
        let b = run_match(7, 1500, all_ai_slots(2));
        assert_eq!(
            a.factions.factions, b.factions.factions,
            "同种子同命令序列必须得到相同终局"
        );
        assert_eq!(
            (a.spawned, a.destroyed, a.captured),
            (b.spawned, b.destroyed, b.captured)
        );
    }
}
