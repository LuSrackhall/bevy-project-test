//! 回归测试：进局瞬间世界刚被重建（单位数为 0）时，胜负判定不得开火。
//!
//! 真实故障（N=4 实测，2026-09-13）：`reset_game_system` 清空并重建世界后的若干帧内，
//! `check_victory_system` 看到"我方无单位"→ 立刻切 `GameOver` → `OnExit(Playing)`
//! 触发 `cleanup_playing_system`（`game_active=false`、driver 被换回 `Live`），
//! 该客户端从此停在局前状态（tick=0、0 士兵），表现为"开局即掉出/卡住"。
//!
//! 这是所有人数规模共有的竞态：N 越大越容易输掉这个时序赌局。

use bevy::prelude::*;
use bevy_adapter::tick::{SimulationWorld, TickClock};
use render_view::{check_victory_system, victory_verdict, GameState};

/// 最小 App：只装仿真世界 + TickClock + 待测系统。
fn app_with_sim(seed: u64, with_map: bool, tick: u32) -> App {
    let mut sim = simulation::init_simulation_world(seed);
    if with_map {
        simulation::map::generate_map(&mut sim, simulation::map::MapSize::Small);
    }
    let mut app = App::new();
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_state(GameState::Playing);
    app.insert_non_send(SimulationWorld::new(sim));
    app.insert_resource(TickClock {
        current_tick: tick,
        ..Default::default()
    });
    app.add_systems(Update, check_victory_system);
    app
}

fn current_state(app: &App) -> GameState {
    app.world().resource::<State<GameState>>().get().clone()
}

/// 本帧系统请求的下一个状态（`None` = 未请求迁移）。
///
/// 必须看 `NextState` 而不是 `State`：状态迁移在 `StateTransition` 调度里生效，
/// 只 `update()` 一次时 `State` 还是旧值，断言会空转通过。
fn requested_state(app: &App) -> Option<GameState> {
    match app.world().resource::<NextState<GameState>>() {
        NextState::Pending(s) | NextState::PendingIfNeq(s) => Some(s.clone()),
        NextState::Unchanged => None,
    }
}

/// 重置后的空世界（tick=0）不得被判 GameOver —— 这正是导致"开局即掉出"的那次误判。
#[test]
fn test_empty_world_at_reset_must_not_be_game_over() {
    let mut app = app_with_sim(42, false, 0);
    app.update();
    assert_ne!(
        requested_state(&app),
        Some(GameState::GameOver),
        "重置瞬间的空世界不得判 GameOver（世界未就绪 ≠ 我方被消灭）"
    );
}

/// 完全无单位的世界（即使 tick 已推进）也不得判负：没有任何单位属于"未就绪"。
#[test]
fn test_unitless_world_is_not_a_loss() {
    let mut app = app_with_sim(42, false, 5);
    app.update();
    assert_ne!(
        requested_state(&app),
        Some(GameState::GameOver),
        "无任何单位的世界不得判负"
    );
    assert_eq!(
        current_state(&app),
        GameState::Playing,
        "且状态应保持在 Playing"
    );
}

// ── 判据纯函数的行为矩阵（含"真判定不得被关掉"的反向守卫）──────────────

/// 世界未就绪（双方都无单位）⇒ 不下结论。
#[test]
fn test_verdict_not_ready_when_no_units_at_all() {
    assert!(!victory_verdict(false, false, 5, true));
    assert!(!victory_verdict(false, false, 0, true));
}

/// 尚未仿真（tick=0）⇒ 不下结论。
#[test]
fn test_verdict_waits_for_first_tick() {
    assert!(!victory_verdict(true, false, 0, true), "tick 0 不得判胜");
    assert!(!victory_verdict(false, true, 0, true), "tick 0 不得判负");
}

/// **已复现的真实故障**：开局早期我方单位尚未出现、敌方已在（tick=1），
/// 但世界还从未"两方俱在"过 ⇒ 不得判负。修复前正是这里误切 GameOver。
#[test]
fn test_verdict_does_not_fire_before_world_has_seen_both_sides() {
    assert!(
        !victory_verdict(false, true, 1, false),
        "尚未见过双方俱在的世界时，不得因我方暂时为空而判负"
    );
    assert!(
        !victory_verdict(true, false, 1, false),
        "同理也不得因敌方暂时为空而判胜"
    );
}

/// 反向守卫：真的见过双方、且一方被清空时，必须判结束（修复不得把真判定一起关掉）。
#[test]
fn test_verdict_still_detects_real_elimination() {
    assert!(victory_verdict(false, true, 5, true), "我方被消灭 ⇒ 结束");
    assert!(victory_verdict(true, false, 5, true), "敌方被消灭 ⇒ 结束（胜利）");
    assert!(!victory_verdict(true, true, 5, true), "双方都有单位 ⇒ 继续");
}
