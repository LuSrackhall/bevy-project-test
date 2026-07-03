//! Architecture tests — compile-time and dependency boundary checks.
//!
//! These tests verify that the architectural invariants from
//! Constitution §1.2.7, §2.5.4, and §2.5.5 are enforced.

use std::path::Path;

fn project_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("Cannot find project root")
}

/// Test: render_view SHALL NOT access simulation::World mutably.
///
/// Constitution §2.5.4.c: "Simulation 外部模块（render_view 等）
/// 永远不得持有 simulation::World 的可写引用"
///
/// Checks for `world_mut()` calls in render_view source.
/// (NonSendMut<SimulationWorld> with query() + submit_command() is safe;
/// world_mut() is the actual violation signal.)
#[test]
fn test_render_view_no_world_mut() {
    let files_to_check = [
        "crates/render_view/src/selection.rs",
        "crates/render_view/src/camera.rs",
        "crates/render_view/src/debug_shape.rs",
        "crates/render_view/src/unit_info_bar.rs",
        "crates/render_view/src/lib.rs",
        "crates/render_view/src/ui/hud.rs",
        "crates/render_view/src/ui/mod.rs",
        "crates/render_view/src/ui/replay_player.rs",
    ];

    let root = project_root();
    let mut violations = Vec::new();

    for rel_path in &files_to_check {
        let file_path = root.join(rel_path);
        if !file_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&file_path).unwrap_or_default();
        // world_mut() gives &mut simulation::World — forbidden in render_view
        // set_world() is the allowed replacement for seek reinit
        for (i, line) in content.lines().enumerate() {
            if line.contains("world_mut(") && !line.contains("set_world(") {
                violations.push(format!("{}:{}: uses world_mut()", rel_path, i + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "ARCHITECTURE VIOLATION — Constitution §2.5.4.c:\n\
         render_view files hold mutable access to simulation::World:\n{:#?}\n\n\
         Use NonSend<SimulationWorld> + query() for reads,\n\
         set_world() for seek reinit, submit_command() for writes.",
        violations
    );
}

/// Test: render_view SHALL NOT directly import simulation::World.
///
/// Constitution §1.2.7 (Simulation 零感知原则)
#[test]
fn test_render_view_no_direct_simulation_import() {
    let files_to_check = [
        "crates/render_view/src/selection.rs",
        "crates/render_view/src/camera.rs",
        "crates/render_view/src/debug_shape.rs",
        "crates/render_view/src/unit_info_bar.rs",
        "crates/render_view/src/lib.rs",
        "crates/render_view/src/ui/hud.rs",
        "crates/render_view/src/ui/mod.rs",
        "crates/render_view/src/ui/replay_player.rs",
    ];

    let root = project_root();
    let mut violations = Vec::new();

    for rel_path in &files_to_check {
        let file_path = root.join(rel_path);
        if !file_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&file_path).unwrap_or_default();
        for (i, line) in content.lines().enumerate() {
            // simulation::World import is forbidden.
            // Component/type/config imports are allowed for queries.
            if line.contains("use simulation::World")
                || line.contains("use simulation::world;")
            {
                violations.push(format!("{}:{}: {}", rel_path, i + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "ARCHITECTURE VIOLATION — Constitution §1.2.7:\n\
         render_view files directly import simulation types:\n{:#?}\n\n\
         Use bevy_adapter's SimulationReader/CommandSink traits.",
        violations
    );
}
