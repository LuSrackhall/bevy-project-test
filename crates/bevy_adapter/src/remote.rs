//! BRP（Bevy Remote Protocol）运行时可观测 —— 仅在 `remote` feature 下编译。
//!
//! ## 为什么不用 BRP 内置的 `world.query`
//!
//! BRP 的内置查询依赖 `Reflect` 类型注册表。但 `simulation` 的组件**不能**派生
//! `bevy_reflect::Reflect`：宪法 §1.4 只允许 `simulation` 引入白名单内的 `bevy_ecs`
//! 子集，其余 `bevy_*` 依赖一律禁止。按 §1 的分层语义，仿真状态对外的投影本就属于
//! 适配层职责，因此这里用**自定义 BRP 方法**暴露一份确定性 JSON 快照，而不是让
//! 仿真层为「可观测性」付出耦合代价。
//!
//! ## agent 用法
//!
//! ```bash
//! # 启动客户端（BRP 默认监听 127.0.0.1:15702）
//! cargo run --features remote -- --windowed
//!
//! # 查询运行中的仿真状态
//! curl -s -X POST http://127.0.0.1:15702/ -H 'content-type: application/json' \
//!   -d '{"jsonrpc":"2.0","id":1,"method":"city_conquest/probe","params":{}}'
//! ```
//!
//! 同一端口同时提供官方内置方法（`world.query`、`world.trigger_event`、
//! 生成 `Screenshot` 实体等），可用于**无人类介入的 UI 验收**：
//! 找到按钮 → 注入点击事件 → 截图 → 断言像素。这是 Bevy 侧唯一有官方支持的
//! UI 自动化路径（官方 `examples/remote/integration_test.rs` 即此流程）。

use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult, RemotePlugin};
use serde_json::{json, Value};

use crate::driver::TickClock;
use crate::tick::SimulationWorld;

/// 自定义方法名。`<domain>/<method>` 与官方内置方法的命名风格一致。
pub const BRP_PROBE_METHOD: &str = "city_conquest/probe";

/// 自定义方法名：请求一张主窗口截图并落盘（agent 据此"看见"UI）。
pub const BRP_SCREENSHOT_METHOD: &str = "city_conquest/screenshot";

/// 构造用于挂载 BRP 的插件（含仿真探针与截图方法）。
pub fn plugin() -> RemotePlugin {
    RemotePlugin::default()
        .with_method_main(BRP_PROBE_METHOD, probe)
        .with_method_main(BRP_SCREENSHOT_METHOD, screenshot)
}

/// 请求主窗口截图落盘。
///
/// 参数：`{"path": "/tmp/shot.png"}`（可选，默认 `/tmp/city-conquest-shot.png`）。
/// 返回：`{"requested": true, "path": ...}`。
///
/// 截图在窗口完成渲染后写出（通常 1–3 帧），因此调用方应轮询文件出现，
/// 而不是把返回值当成"已完成"。这是 Bevy 侧唯一有官方支持的 UI 可视化路径，
/// 让 agent 能对"UI 长什么样"做机器可读的断言（读取 PNG 后比对/审阅）。
fn screenshot(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    use bevy::render::view::screenshot::{save_to_disk, Screenshot};

    let path = params
        .as_ref()
        .and_then(|p| p.get("path"))
        .and_then(|v| v.as_str())
        .unwrap_or("/tmp/city-conquest-shot.png")
        .to_string();

    // 官方用法：spawn 一个 Screenshot 实体，并挂上 save_to_disk 观察者。
    world
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path.clone()));

    Ok(json!({ "requested": true, "path": path }))
}

/// 仿真运行时快照。
///
/// 返回字段：
/// - `tick` —— 调度层当前 Tick（`TickClock.current_tick`）
/// - `world_hash` —— 与 `sim-cli` 同源的黄金哈希（`golden_test::hash_world_state`）
/// - `total_soldiers` / `total_cities`
/// - `factions: [{faction, soldiers, cities}]`，按 faction 升序（确定性）
///
/// `world_hash` 与 `sim-cli` 完全同源，因此可以拿**运行中的客户端**与**无头回放**
/// 直接对比哈希，用于定位 desync 属于「仿真分歧」还是「表现层问题」。
pub fn probe(_: In<Option<Value>>, world: &mut World) -> BrpResult {
    let tick = world
        .get_resource::<TickClock>()
        .map(|clock| clock.current_tick)
        .unwrap_or(0);

    let Some(mut sim_world) = world.get_non_send_mut::<SimulationWorld>() else {
        return Err(BrpError::internal(
            "SimulationWorld 尚未初始化（对局未开始，或仍处于大厅阶段）",
        ));
    };

    let counts = simulation::world_stats::count_factions(sim_world.world_mut());
    let hash = simulation::golden_test::hash_world_state(sim_world.world_mut());

    let factions: Vec<Value> = counts
        .factions
        .iter()
        .map(|(faction, (soldiers, cities))| {
            json!({"faction": faction.0, "soldiers": soldiers, "cities": cities})
        })
        .collect();

    Ok(json!({
        "tick": tick,
        "world_hash": hash,
        "total_soldiers": counts.total_soldiers(),
        "total_cities": counts.total_cities(),
        "factions": factions,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造一个跑过 `ticks` 个 Tick 的 Bevy World（内含仿真世界）。
    fn world_with_sim(seed: u64, ticks: u32) -> World {
        let mut sim = simulation::init_simulation_world(seed);
        simulation::map::generate_map(&mut sim, simulation::map::MapSize::Small);
        let config = simulation::RunConfig { enable_ai: true };
        for tick in 1..=ticks {
            simulation::run_tick(&mut sim, tick, &config);
        }

        let mut world = World::new();
        world.insert_resource(TickClock {
            current_tick: ticks,
            ..Default::default()
        });
        world.insert_non_send(SimulationWorld::new(sim));
        world
    }

    /// 探针必须报出可判定的仿真状态（agent 靠它断言运行中的客户端）。
    #[test]
    fn probe_reports_simulation_state() {
        let mut world = world_with_sim(42, 400);
        let value = probe(In(None), &mut world).expect("探针必须成功");

        assert_eq!(value["tick"], 400);
        assert!(value["world_hash"].as_u64().unwrap_or(0) > 0);
        assert!(value["total_cities"].as_u64().unwrap_or(0) > 0);

        let factions = value["factions"].as_array().expect("factions 必须是数组");
        assert!(
            factions.len() >= 2,
            "至少应有玩家与敌方两个阵营：{factions:?}"
        );
    }

    /// 哈希必须与无头路径同源 —— 否则「客户端 vs 回放」的对比无意义。
    #[test]
    fn probe_hash_matches_headless_hash() {
        let mut world = world_with_sim(7, 300);
        let reported = probe(In(None), &mut world).unwrap()["world_hash"]
            .as_u64()
            .unwrap();

        let expected = {
            let mut sim = world
                .get_non_send_mut::<SimulationWorld>()
                .expect("仿真世界存在");
            simulation::golden_test::hash_world_state(sim.world_mut())
        };

        assert_eq!(reported, expected, "BRP 探针与无头路径必须报同一哈希");
    }

    /// 缺仿真世界时快速失败并说明原因，而不是给出误导性的空快照。
    #[test]
    fn probe_without_simulation_world_fails_fast() {
        let mut world = World::new();
        world.insert_resource(TickClock::default());

        let err = probe(In(None), &mut world).expect_err("缺少仿真世界必须报错");
        assert!(
            err.message.contains("SimulationWorld"),
            "错误信息应指明原因，实际：{}",
            err.message
        );
    }

    /// 插件可构造（方法注册在 App 构建期完成）。
    #[test]
    fn plugin_is_constructible() {
        let _ = plugin();
    }
}
