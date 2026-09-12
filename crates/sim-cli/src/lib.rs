//! sim-cli —— 无头、面向 agent 的仿真验证入口。
//!
//! 存在理由：`simulation` 的 212 个测试与 scenario harness 只能从 Rust 测试里调用，
//! 于是"验证一次改动"必须由人写 Rust。本 CLI 把它变成
//! `sim-cli scenario --json` + 退出码，让 agent 能不写 Rust 就闭环。
//!
//! 宪法约束：
//! - §21  独立二进制，不进入 `simulation` 依赖图；**不依赖 bevy 渲染栈**（迭代编译与渲染解耦）
//! - §3.1 所有 tick 经 `run_tick`；命令只经 `CommandBuffer` 注入
//! - §10  内置确定性门（同参重复运行哈希一致）、回放门（录制→重放逐检查点哈希一致）、黄金哈希断言
//! - §20.2 回放格式版本不兼容时快速失败，不静默降级
//!
//! 退出码：`0` 通过 / `1` 验证失败 / `2` 用法、IO 或格式错误。

use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};
use simulation::command::CommandBuffer;
use simulation::events::SimulationEvents;
use simulation::map::{self, MapSize};
use simulation::replay::ReplayFile;
use simulation::types::{
    AiProfile, Controller, FactionId, PlayerSlot, PlayerSlots, SlotId, TeamId,
};
use simulation::world_stats::{count_factions, FactionCounts};
use simulation::RunConfig;
use simulation::{golden_test, init_simulation_world, init_simulation_world_multi, run_tick};

/// 中立阵营（content 的地图配置以 `neutral_city_ratio` 生成，AI 亦按 FactionId(2) 识别）。
pub const NEUTRAL_FACTION: u8 = 2;

pub const EXIT_PASS: i32 = 0;
pub const EXIT_FAIL: i32 = 1;
pub const EXIT_ERROR: i32 = 2;

// ───────────────────────────── 错误 ─────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum CliError {
    Usage(String),
    Io(String),
    Format(String),
}

impl CliError {
    pub fn message(&self) -> String {
        match self {
            CliError::Usage(m) => format!("用法错误：{m}"),
            CliError::Io(m) => format!("IO 错误：{m}"),
            CliError::Format(m) => format!("格式错误：{m}"),
        }
    }
    pub fn exit_code(&self) -> i32 {
        EXIT_ERROR
    }
}

// ───────────────────────────── 参数 ─────────────────────────────

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ScenarioArgs {
    pub seed: u64,
    pub map: MapSize,
    pub ticks: u32,
    pub enable_ai: bool,
    pub expect_hash: Option<u64>,
    pub record: Option<PathBuf>,
    pub verify_replay: bool,
    pub require_decided: bool,
    pub repeat: u32,
    pub json: bool,
    pub quiet: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReplayArgs {
    pub file: PathBuf,
    pub enable_ai: bool,
    pub expect_final_hash: Option<u64>,
    pub json: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SelfPlayArgs {
    pub seed: u64,
    pub map: MapSize,
    pub ticks: u32,
    pub players: u8,
    /// 所有槽位都由 AI 控制（真·对称自对弈）；否则 faction 0 被动、其余为 AI。
    pub symmetric: bool,
    pub require_decided: bool,
    /// 要求对局必须在此时刻（tick）之前决出；给出即隐含 `require_decided`。
    pub decide_by: Option<u32>,
    pub json: bool,
    pub quiet: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Scenario(ScenarioArgs),
    Replay(ReplayArgs),
    SelfPlay(SelfPlayArgs),
    Help,
}

pub fn parse(argv: &[String]) -> Result<Command, CliError> {
    let mut it = argv.iter().skip(1);
    let sub = match it.next() {
        Some(s) => s.as_str(),
        None => return Ok(Command::Help),
    };
    let rest: Vec<String> = it.cloned().collect();
    match sub {
        "scenario" => Ok(Command::Scenario(parse_scenario(&rest)?)),
        "replay" => Ok(Command::Replay(parse_replay(&rest)?)),
        "selfplay" => Ok(Command::SelfPlay(parse_selfplay(&rest)?)),
        "--help" | "-h" | "help" => Ok(Command::Help),
        other => Err(CliError::Usage(format!(
            "未知子命令 `{other}`（可用：scenario | replay | selfplay | help）"
        ))),
    }
}

/// 把 `--key value` / `--key=value` / 裸 `--flag` 统一成 map；非 `--` 开头者为位置参数。
fn scan(rest: &[String]) -> (HashMap<String, String>, Vec<String>) {
    let mut flags = HashMap::new();
    let mut positional = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        let arg = &rest[i];
        if let Some(body) = arg.strip_prefix("--") {
            if let Some((k, v)) = body.split_once('=') {
                flags.insert(k.to_string(), v.to_string());
            } else {
                match rest.get(i + 1) {
                    Some(v) if !v.starts_with("--") => {
                        flags.insert(body.to_string(), v.clone());
                        i += 1;
                    }
                    _ => {
                        flags.insert(body.to_string(), "true".to_string());
                    }
                }
            }
        } else {
            positional.push(arg.clone());
        }
        i += 1;
    }
    (flags, positional)
}

fn num<T: std::str::FromStr>(
    flags: &HashMap<String, String>,
    key: &str,
    default: T,
) -> Result<T, CliError> {
    match flags.get(key) {
        None => Ok(default),
        Some(raw) => raw
            .parse::<T>()
            .map_err(|_| CliError::Usage(format!("--{key} 的值 `{raw}` 无法解析"))),
    }
}

fn bool_of(flags: &HashMap<String, String>, key: &str, default: bool) -> Result<bool, CliError> {
    match flags.get(key) {
        None => Ok(default),
        Some(raw) => match raw.to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            other => Err(CliError::Usage(format!(
                "--{key} 期望布尔值，收到 `{other}`"
            ))),
        },
    }
}

fn map_of(
    flags: &HashMap<String, String>,
    key: &str,
    default: MapSize,
) -> Result<MapSize, CliError> {
    match flags.get(key) {
        None => Ok(default),
        Some(raw) => match raw.to_ascii_lowercase().as_str() {
            "small" => Ok(MapSize::Small),
            "medium" => Ok(MapSize::Medium),
            "large" => Ok(MapSize::Large),
            "huge" => Ok(MapSize::Huge),
            other => Err(CliError::Usage(format!(
                "--{key} 期望 small|medium|large|huge，收到 `{other}`"
            ))),
        },
    }
}

fn parse_scenario(rest: &[String]) -> Result<ScenarioArgs, CliError> {
    let (flags, positional) = scan(rest);
    if !positional.is_empty() {
        return Err(CliError::Usage(format!(
            "scenario 不接受位置参数，收到 {:?}",
            positional
        )));
    }
    Ok(ScenarioArgs {
        seed: num(&flags, "seed", 42u64)?,
        map: map_of(&flags, "map", MapSize::Small)?,
        ticks: num(&flags, "ticks", 500u32)?,
        enable_ai: !bool_of(&flags, "no-ai", false)?,
        expect_hash: match flags.get("expect-hash") {
            None => None,
            Some(v) => Some(
                v.parse()
                    .map_err(|_| CliError::Usage(format!("--expect-hash 的值 `{v}` 不是 u64")))?,
            ),
        },
        record: flags.get("record").map(PathBuf::from),
        verify_replay: bool_of(&flags, "verify-replay", false)?,
        require_decided: bool_of(&flags, "require-decided", false)?,
        repeat: num(&flags, "repeat", 2u32)?,
        json: bool_of(&flags, "json", false)?,
        quiet: bool_of(&flags, "quiet", false)?,
    })
}

fn parse_replay(rest: &[String]) -> Result<ReplayArgs, CliError> {
    let (flags, positional) = scan(rest);
    let file = match positional.first() {
        Some(p) => PathBuf::from(p),
        None => match flags.get("file") {
            Some(p) => PathBuf::from(p),
            None => {
                return Err(CliError::Usage(
                    "replay 需要一个回放文件路径：`sim-cli replay <file.ron>`".to_string(),
                ))
            }
        },
    };
    Ok(ReplayArgs {
        file,
        enable_ai: !bool_of(&flags, "no-ai", false)?,
        expect_final_hash: match flags.get("expect-final-hash") {
            None => None,
            Some(v) => Some(v.parse().map_err(|_| {
                CliError::Usage(format!("--expect-final-hash 的值 `{v}` 不是 u64"))
            })?),
        },
        json: bool_of(&flags, "json", false)?,
    })
}

fn parse_selfplay(rest: &[String]) -> Result<SelfPlayArgs, CliError> {
    let (flags, positional) = scan(rest);
    if !positional.is_empty() {
        return Err(CliError::Usage(format!(
            "selfplay 不接受位置参数，收到 {:?}",
            positional
        )));
    }
    let players: u8 = num(&flags, "players", 2u8)?;
    if players < 2 {
        return Err(CliError::Usage("--players 至少为 2".to_string()));
    }
    Ok(SelfPlayArgs {
        seed: num(&flags, "seed", 42u64)?,
        map: map_of(&flags, "map", MapSize::Small)?,
        ticks: num(&flags, "ticks", 6000u32)?,
        players,
        symmetric: bool_of(&flags, "symmetric", false)?,
        require_decided: bool_of(&flags, "require-decided", false)?,
        decide_by: match flags.get("decide-by") {
            None => None,
            Some(v) => Some(
                v.parse()
                    .map_err(|_| CliError::Usage(format!("--decide-by 的值 `{v}` 不是 u32")))?,
            ),
        },
        json: bool_of(&flags, "json", false)?,
        quiet: bool_of(&flags, "quiet", false)?,
    })
}

// ───────────────────────────── 结果 ─────────────────────────────

#[derive(Debug)]
pub struct Outcome {
    pub ok: bool,
    pub json: Value,
    pub text: String,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventTotals {
    pub spawned: u64,
    pub destroyed: u64,
    pub captured: u64,
    pub damage: u64,
    pub leveled_up: u64,
}

impl EventTotals {
    fn absorb(&mut self, ev: &SimulationEvents) {
        self.spawned += ev.spawned.len() as u64;
        self.destroyed += ev.destroyed.len() as u64;
        self.captured += ev.captured.len() as u64;
        self.damage += ev.damage.len() as u64;
        self.leveled_up += ev.leveled_up.len() as u64;
    }
    fn json(&self) -> Value {
        json!({
            "spawned": self.spawned,
            "destroyed": self.destroyed,
            "captured": self.captured,
            "damage": self.damage,
            "leveled_up": self.leveled_up,
        })
    }
    fn text(&self) -> String {
        format!(
            "spawned:{} destroyed:{} captured:{} damage:{} leveled_up:{}",
            self.spawned, self.destroyed, self.captured, self.damage, self.leveled_up
        )
    }
}

/// 一个 faction 都不剩时才算决出胜负；中立（FactionId(2)）不计入竞争者。
fn decided_winner(counts: &FactionCounts) -> Option<u8> {
    let contested: Vec<u8> = counts
        .factions
        .iter()
        .filter(|(f, (_, cities))| f.0 != NEUTRAL_FACTION && *cities > 0)
        .map(|(f, _)| f.0)
        .collect();
    if contested.len() == 1 {
        Some(contested[0])
    } else {
        None
    }
}

fn factions_json(counts: &FactionCounts) -> Value {
    Value::Array(
        counts
            .factions
            .iter()
            .map(|(f, (soldiers, cities))| {
                json!({"faction": f.0, "soldiers": soldiers, "cities": cities})
            })
            .collect(),
    )
}

fn factions_text(counts: &FactionCounts) -> String {
    counts
        .factions
        .iter()
        .map(|(f, (s, c))| format!("{}:{{soldiers:{},cities:{}}}", f.0, s, c))
        .collect::<Vec<_>>()
        .join(" ")
}

struct RunResult {
    final_hash: u64,
    counts: FactionCounts,
    events: EventTotals,
    replay: Option<ReplayFile>,
    /// 首次采样到"只剩一个非中立阵营持有城池"的 tick（分辨率 = DESYNC_CHECK_INTERVAL）。
    decided_at: Option<u32>,
}

/// 执行一次确定性仿真：seed + map + ticks（+ 可选回放录制）。
fn execute(
    seed: u64,
    map_size: MapSize,
    ticks: u32,
    enable_ai: bool,
    slots: Option<PlayerSlots>,
    record: bool,
) -> RunResult {
    let mut world = match slots {
        Some(s) => init_simulation_world_multi(seed, s),
        None => init_simulation_world(seed),
    };
    map::generate_map(&mut world, map_size);
    if record {
        world.insert_resource(ReplayFile::new(seed, map_size, ticks));
    }

    let config = RunConfig { enable_ai };
    let mut events = EventTotals::default();
    let mut decided_at: Option<u32> = None;
    for tick in 1..=ticks {
        let ev = run_tick(&mut world, tick, &config);
        events.absorb(&ev);
        if tick % ReplayFile::DESYNC_CHECK_INTERVAL == 0 {
            // 采样"是否已决出"：给对局时长一个可量化指标（分辨率 20 tick）
            if decided_at.is_none() && decided_winner(&count_factions(&mut world)).is_some() {
                decided_at = Some(tick);
            }
            if world.contains_resource::<ReplayFile>() {
                let hash = golden_test::hash_world_state(&mut world);
                if let Some(mut recorder) = world.get_resource_mut::<ReplayFile>() {
                    recorder.record_tick_hash(tick, hash);
                }
            }
        }
    }

    let final_hash = golden_test::hash_world_state(&mut world);
    let counts = count_factions(&mut world);
    RunResult {
        final_hash,
        counts,
        events,
        replay: world.remove_resource::<ReplayFile>(),
        decided_at,
    }
}

/// 回放/重放一次的完整结果（避免多元组，满足 clippy::type_complexity）。
struct ReplayComparison {
    final_hash: u64,
    counts: FactionCounts,
    events: EventTotals,
    checkpoints_compared: usize,
    mismatches: Vec<(u32, u64, u64)>,
}

/// 从回放文件重放并逐检查点比对哈希（§10.1 回放测试）。
fn replay_and_compare(replay: &ReplayFile, enable_ai: bool) -> ReplayComparison {
    let mut world = init_simulation_world(replay.seed);
    map::generate_map(&mut world, replay.map_size);
    let config = RunConfig { enable_ai };

    let mut events = EventTotals::default();
    let mut checked = 0usize;
    let mut mismatches: Vec<(u32, u64, u64)> = Vec::new();

    for tick in 1..=replay.total_ticks {
        for cmd in replay.commands_for_tick(tick) {
            world.resource_mut::<CommandBuffer>().push(cmd.clone());
        }
        let ev = run_tick(&mut world, tick, &config);
        events.absorb(&ev);

        if tick % ReplayFile::DESYNC_CHECK_INTERVAL == 0 {
            if let Some(expected) = replay.hash_for_tick(tick) {
                let actual = golden_test::hash_world_state(&mut world);
                checked += 1;
                if expected != actual {
                    mismatches.push((tick, expected, actual));
                }
            }
        }
    }

    let final_hash = golden_test::hash_world_state(&mut world);
    let counts = count_factions(&mut world);
    ReplayComparison {
        final_hash,
        counts,
        events,
        checkpoints_compared: checked,
        mismatches,
    }
}

// ───────────────────────────── 子命令 ─────────────────────────────

fn run_scenario(args: &ScenarioArgs) -> Result<Outcome, CliError> {
    let mut failures: Vec<String> = Vec::new();
    let mut hashes: Vec<u64> = Vec::new();
    let mut last: Option<RunResult> = None;

    let want_record = args.record.is_some();
    for _ in 0..args.repeat.max(1) {
        let result = execute(
            args.seed,
            args.map,
            args.ticks,
            args.enable_ai,
            None,
            want_record,
        );
        hashes.push(result.final_hash);
        last = Some(result);
    }
    let last = last.expect("repeat >= 1 保证至少一次执行");

    // §10.1 确定性门
    let deterministic = hashes.windows(2).all(|w| w[0] == w[1]);
    if !deterministic {
        failures.push(format!(
            "确定性失败：同一输入重复运行得到不同哈希 {:?}",
            hashes
        ));
    }

    // §10.2 黄金哈希断言
    let mut hash_matched = None;
    if let Some(expected) = args.expect_hash {
        let matched = last.final_hash == expected;
        hash_matched = Some(matched);
        if !matched {
            failures.push(format!(
                "黄金哈希不匹配：expected={expected} actual={}",
                last.final_hash
            ));
        }
    }

    // 对局是否决出胜负
    let winner = decided_winner(&last.counts);
    if args.require_decided && winner.is_none() {
        failures.push(format!(
            "{} tick 内未决出胜负（各阵营城池：{}）",
            args.ticks,
            factions_text(&last.counts)
        ));
    }

    // 录制 → 重放一致性（录制时总是顺带验证）
    let mut replay_check: Option<Value> = None;
    if let (Some(path), Some(replay)) = (&args.record, last.replay.as_ref()) {
        fs::write(path, replay.to_ron())
            .map_err(|e| CliError::Io(format!("写入回放 {} 失败：{e}", path.display())))?;
        let _ = args.verify_replay; // 录制即验证（P0 要求），显式传参只作语义强调
        let ReplayComparison {
            final_hash,
            counts,
            checkpoints_compared: checked,
            mismatches,
            ..
        } = replay_and_compare(replay, args.enable_ai);
        if final_hash != last.final_hash {
            failures.push(format!(
                "回放终态哈希不一致：recorded={} replayed={final_hash}",
                last.final_hash
            ));
        }
        if !mismatches.is_empty() {
            failures.push(format!(
                "回放检查点哈希不一致 {} 处，首个 tick={} expected={} actual={}",
                mismatches.len(),
                mismatches[0].0,
                mismatches[0].1,
                mismatches[0].2
            ));
        }
        replay_check = Some(json!({
            "file": path.display().to_string(),
            "checkpoints_compared": checked,
            "mismatches": mismatches.len(),
            "final_hash_replayed": final_hash,
            "final_factions_replayed": factions_json(&counts),
        }));
    } else if args.verify_replay {
        failures.push("--verify-replay 需要同时给出 --record <path>".to_string());
    }

    let ok = failures.is_empty();
    let json = json!({
        "subcommand": "scenario",
        "seed": args.seed,
        "map": format!("{:?}", args.map).to_lowercase(),
        "ticks": args.ticks,
        "ai": args.enable_ai,
        "runs": args.repeat.max(1),
        "determinism": {
            "stable": deterministic,
            "hashes": hashes,
        },
        "golden_hash": {
            "expected": args.expect_hash,
            "matched": hash_matched,
        },
        "final_hash": last.final_hash,
        "factions": factions_json(&last.counts),
        "events": last.events.json(),
        "decided_winner": winner,
        "replay": replay_check,
        "verdict": if ok { "pass" } else { "fail" },
        "failures": failures,
    });

    let mut text = String::new();
    if !args.quiet {
        let _ = writeln!(
            text,
            "scenario seed={} map={:?} ticks={} ai={} runs={}",
            args.seed,
            args.map,
            args.ticks,
            args.enable_ai,
            args.repeat.max(1)
        );
        let _ = writeln!(text, "  final_hash   = {}", last.final_hash);
        let _ = writeln!(
            text,
            "  determinism  = {}",
            if deterministic { "stable" } else { "UNSTABLE" }
        );
        let _ = writeln!(text, "  factions     = {}", factions_text(&last.counts));
        let _ = writeln!(text, "  events       = {}", last.events.text());
        let _ = writeln!(text, "  decided      = {:?}", winner);
        if let Some(rc) = &replay_check {
            let _ = writeln!(
                text,
                "  replay       = {} checkpoints, {} mismatches",
                rc["checkpoints_compared"], rc["mismatches"]
            );
        }
        for f in &failures {
            let _ = writeln!(text, "  FAIL: {f}");
        }
        let _ = writeln!(
            text,
            "  verdict      = {}",
            if ok { "PASS" } else { "FAIL" }
        );
    } else {
        let _ = writeln!(text, "{}", last.final_hash);
    }

    Ok(Outcome { ok, json, text })
}

fn run_replay(args: &ReplayArgs) -> Result<Outcome, CliError> {
    let text_in = fs::read_to_string(&args.file)
        .map_err(|e| CliError::Io(format!("读取回放 {} 失败：{e}", args.file.display())))?;
    // §20.2：版本不兼容必须快速失败
    let replay = ReplayFile::from_ron(&text_in).map_err(CliError::Format)?;

    let ReplayComparison {
        final_hash,
        counts,
        events,
        checkpoints_compared: checked,
        mismatches,
    } = replay_and_compare(&replay, args.enable_ai);

    let mut failures: Vec<String> = Vec::new();
    let has_checkpoints = !replay.tick_hashes.is_empty();
    if !mismatches.is_empty() {
        failures.push(format!(
            "回放 desync：{} 个检查点哈希不一致，首个 tick={} expected={} actual={}",
            mismatches.len(),
            mismatches[0].0,
            mismatches[0].1,
            mismatches[0].2
        ));
    }
    let mut final_matched = None;
    if let Some(expected) = args.expect_final_hash {
        let matched = expected == final_hash;
        final_matched = Some(matched);
        if !matched {
            failures.push(format!(
                "终态哈希不匹配：expected={expected} actual={final_hash}"
            ));
        }
    }

    let ok = failures.is_empty();
    let json = json!({
        "subcommand": "replay",
        "file": args.file.display().to_string(),
        "format_version": replay.format_version,
        "seed": replay.seed,
        "map": format!("{:?}", replay.map_size).to_lowercase(),
        "total_ticks": replay.total_ticks,
        "ai": args.enable_ai,
        "commands_replayed": replay.commands_per_tick.values().map(|v| v.len() as u64).sum::<u64>(),
        "hash_checkpoints": {
            "declared": replay.tick_hashes.len(),
            "compared": checked,
            "mismatches": mismatches.iter().map(|(t, e, a)| json!({"tick": t, "expected": e, "actual": a})).collect::<Vec<_>>(),
            "note": if has_checkpoints { Value::Null } else { json!("该回放未记录 tick_hashes，仅比对终态哈希") },
        },
        "final_hash": final_hash,
        "final_hash_expected": args.expect_final_hash,
        "final_hash_matched": final_matched,
        "factions": factions_json(&counts),
        "events": events.json(),
        "verdict": if ok { "pass" } else { "fail" },
        "failures": failures,
    });

    let mut text = String::new();
    let _ = writeln!(
        text,
        "replay {} v{} seed={} map={:?} ticks={} ai={}",
        args.file.display(),
        replay.format_version,
        replay.seed,
        replay.map_size,
        replay.total_ticks,
        args.enable_ai
    );
    let _ = writeln!(
        text,
        "  checkpoints  = {} declared / {} compared / {} mismatches",
        replay.tick_hashes.len(),
        checked,
        mismatches.len()
    );
    let _ = writeln!(text, "  final_hash   = {final_hash}");
    let _ = writeln!(text, "  factions     = {}", factions_text(&counts));
    if !has_checkpoints {
        let _ = writeln!(
            text,
            "  note         = 回放未含 tick_hashes，仅验证终态哈希"
        );
    }
    for f in &failures {
        let _ = writeln!(text, "  FAIL: {f}");
    }
    let _ = writeln!(
        text,
        "  verdict      = {}",
        if ok { "PASS" } else { "FAIL" }
    );

    Ok(Outcome { ok, json, text })
}

/// 构建 AI 对被动玩家的槽位：faction 0 为人类（不产生命令），其余为 AI。
///
/// 构建 selfplay 的槽位配置。
///
/// - `symmetric = true`：全部槽位由 AI 控制 → **真·对称自对弈**
///   （依赖 `simulation::ai` 已泛化敌方选择，不再硬编码 `FactionId(0)`）。
/// - `symmetric = false`：faction 0 为被动人类（不产生命令），其余为 AI。
fn selfplay_slots(players: u8, symmetric: bool) -> PlayerSlots {
    let slots = (0..players)
        .map(|i| PlayerSlot {
            slot_id: SlotId(i),
            controller: if symmetric || i > 0 {
                Controller::AI(AiProfile::default())
            } else {
                Controller::HumanLocal
            },
            faction: FactionId(i),
            team: TeamId(i),
        })
        .collect();
    PlayerSlots { slots }
}

fn run_selfplay(args: &SelfPlayArgs) -> Result<Outcome, CliError> {
    let result = execute(
        args.seed,
        args.map,
        args.ticks,
        true,
        Some(selfplay_slots(args.players, args.symmetric)),
        false,
    );
    let winner = decided_winner(&result.counts);

    let mut failures: Vec<String> = Vec::new();
    // --decide-by 隐含 require_decided，并额外要求"够快"
    if let Some(limit) = args.decide_by {
        match result.decided_at {
            None => failures.push(format!(
                "{} tick 内未决出胜负（各阵营城池：{}）",
                args.ticks,
                factions_text(&result.counts)
            )),
            Some(t) if t > limit => failures.push(format!(
                "对局直到 tick {t} 才决出，超过 --decide-by {limit}（各阵营城池：{}）",
                factions_text(&result.counts)
            )),
            Some(_) => {}
        }
    } else if args.require_decided && winner.is_none() {
        failures.push(format!(
            "{} tick 内未决出胜负（各阵营城池：{}）",
            args.ticks,
            factions_text(&result.counts)
        ));
    }
    let ok = failures.is_empty();

    let ai_factions: Vec<u8> = if args.symmetric {
        (0..args.players).collect()
    } else {
        (1..args.players).collect()
    };
    let passive_factions: Vec<u8> = if args.symmetric { vec![] } else { vec![0] };

    let json = json!({
        "subcommand": "selfplay",
        "seed": args.seed,
        "map": format!("{:?}", args.map).to_lowercase(),
        "max_ticks": args.ticks,
        "players": args.players,
        "symmetric": args.symmetric,
        "ai_factions": ai_factions,
        "passive_factions": passive_factions,
        "decided_winner": winner,
        "decided_at": result.decided_at,
        "decide_by": args.decide_by,
        "final_hash": result.final_hash,
        "factions": factions_json(&result.counts),
        "events": result.events.json(),
        "verdict": if ok { "pass" } else { "fail" },
        "failures": failures,
    });

    let mut text = String::new();
    if !args.quiet {
        let _ = writeln!(
            text,
            "selfplay seed={} map={:?} max_ticks={} players={} symmetric={}",
            args.seed, args.map, args.ticks, args.players, args.symmetric
        );
        let _ = writeln!(text, "  final_hash   = {}", result.final_hash);
        let _ = writeln!(text, "  factions     = {}", factions_text(&result.counts));
        let _ = writeln!(text, "  events       = {}", result.events.text());
        let _ = writeln!(
            text,
            "  decided      = {}",
            winner.map_or(
                "未决出（可能 tick 上限不足或 AI 未能推进）".to_string(),
                |w| match result.decided_at {
                    Some(t) => format!("faction {w} @ tick {t}"),
                    None => format!("faction {w}"),
                }
            )
        );
        if !args.symmetric {
            let _ = writeln!(
                text,
                "  note         = 非对称：faction 0 被动，AI 控制其余槽位"
            );
        }
        for f in &failures {
            let _ = writeln!(text, "  FAIL: {f}");
        }
        let _ = writeln!(
            text,
            "  verdict      = {}",
            if ok { "PASS" } else { "FAIL" }
        );
    } else {
        let _ = writeln!(text, "{}", result.final_hash);
    }

    Ok(Outcome { ok, json, text })
}

// ───────────────────────────── 入口 ─────────────────────────────

pub const HELP: &str = "\
sim-cli —— simulation 的无头验证入口（agent 原生闭环）

用法：
  sim-cli scenario [--seed N] [--map small|medium|large|huge] [--ticks N]
                   [--no-ai] [--expect-hash H] [--repeat N]
                   [--record <file.ron>] [--verify-replay]
                   [--require-decided] [--json] [--quiet]

  sim-cli replay <file.ron> [--no-ai] [--expect-final-hash H] [--json]

  sim-cli selfplay [--seed N] [--map SIZE] [--ticks N] [--players N]
                   [--symmetric] [--require-decided] [--decide-by TICKS]
                   [--json] [--quiet]

说明：
  scenario  跑 seed+map+ticks 的确定性仿真。默认 repeat=2（内置确定性门），
            可用 --expect-hash 断言黄金哈希，可用 --record 落盘回放并**立即重放比对**，
            可用 --require-decided 断言对局在 tick 上限内决出胜负。
  replay    重放回放文件，逐 20 tick（DESYNC_CHECK_INTERVAL）比对已记录哈希。
            格式版本不兼容时快速失败（宪法 §20.2）。
  selfplay  AI 对被动方（--symmetric 时为 AI 对 AI 的对称自对弈）跑到 tick 上限，
            报告胜负、阵营统计、决出时刻与终态哈希。
            --require-decided 要求必须决出胜负；--decide-by TICKS 额外要求够快
            （隐含 --require-decided；决出时刻按每 20 tick 采样，分辨率 20 tick）。

退出码：0 通过 / 1 验证失败 / 2 用法、IO 或格式错误。
";

pub fn run(cmd: Command) -> Result<Outcome, CliError> {
    match cmd {
        Command::Scenario(a) => run_scenario(&a),
        Command::Replay(a) => run_replay(&a),
        Command::SelfPlay(a) => run_selfplay(&a),
        Command::Help => Ok(Outcome {
            ok: true,
            json: json!({"subcommand": "help"}),
            text: HELP.to_string(),
        }),
    }
}

pub fn main_with(argv: &[String]) -> i32 {
    let cmd = match parse(argv) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e.message());
            return e.exit_code();
        }
    };
    let json = matches!(
        cmd,
        Command::Scenario(ScenarioArgs { json: true, .. })
            | Command::Replay(ReplayArgs { json: true, .. })
            | Command::SelfPlay(SelfPlayArgs { json: true, .. })
    );

    match run(cmd) {
        Ok(outcome) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&outcome.json).unwrap_or_else(|_| "{}".into())
                );
            } else {
                print!("{}", outcome.text);
            }
            if outcome.ok {
                EXIT_PASS
            } else {
                EXIT_FAIL
            }
        }
        Err(e) => {
            eprintln!("{}", e.message());
            e.exit_code()
        }
    }
}

// ───────────────────────────── 测试 ─────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(s: &str) -> Vec<String> {
        s.split_whitespace().map(|x| x.to_string()).collect()
    }

    #[test]
    fn scenario_defaults_are_verification_oriented() {
        let cmd = parse(&argv("sim-cli scenario")).unwrap();
        let Command::Scenario(a) = cmd else {
            panic!("期望 scenario")
        };
        assert_eq!(a.seed, 42);
        assert_eq!(a.ticks, 500);
        assert!(a.enable_ai);
        assert_eq!(a.repeat, 2, "默认必须内置确定性门");
        assert!(!a.require_decided);
    }

    #[test]
    fn scenario_parses_all_flags() {
        let cmd = parse(&argv(
            "sim-cli scenario --seed 7 --map Huge --ticks 99 --no-ai --expect-hash 123 --record /tmp/x.ron --require-decided --json",
        ))
        .unwrap();
        let Command::Scenario(a) = cmd else { panic!() };
        assert_eq!(a.seed, 7);
        assert_eq!(a.map, MapSize::Huge, "map 名应大小写不敏感");
        assert_eq!(a.ticks, 99);
        assert!(!a.enable_ai);
        assert_eq!(a.expect_hash, Some(123));
        assert_eq!(a.record, Some(PathBuf::from("/tmp/x.ron")));
        assert!(a.require_decided);
        assert!(a.json);
    }

    #[test]
    fn replay_accepts_positional_file() {
        let Command::Replay(a) = parse(&argv("sim-cli replay replays/a.ron")).unwrap() else {
            panic!()
        };
        assert_eq!(a.file, PathBuf::from("replays/a.ron"));
        assert!(a.enable_ai);
    }

    #[test]
    fn replay_requires_a_file() {
        assert!(matches!(
            parse(&argv("sim-cli replay")),
            Err(CliError::Usage(_))
        ));
    }

    #[test]
    fn unknown_subcommand_is_usage_error() {
        assert!(matches!(
            parse(&argv("sim-cli nonsense")),
            Err(CliError::Usage(_))
        ));
    }

    #[test]
    fn selfplay_rejects_single_player() {
        assert!(matches!(
            parse(&argv("sim-cli selfplay --players 1")),
            Err(CliError::Usage(_))
        ));
    }

    #[test]
    fn selfplay_parses_decide_by() {
        let Command::SelfPlay(a) = parse(&argv("sim-cli selfplay --decide-by 3000")).unwrap()
        else {
            panic!("期望 selfplay");
        };
        assert_eq!(a.decide_by, Some(3000));
        assert!(matches!(
            parse(&argv("sim-cli selfplay --decide-by abc")),
            Err(CliError::Usage(_))
        ));
    }

    /// `--decide-by` 是有牙齿的门：预算太小必须 FAIL（用于防止 AI 退化回"不进攻"）。
    #[test]
    fn decide_by_gate_fails_when_budget_too_small() {
        let args = SelfPlayArgs {
            seed: 42,
            map: MapSize::Small,
            ticks: 200, // 远小于决出所需
            players: 2,
            symmetric: true,
            require_decided: false,
            decide_by: Some(200),
            json: true,
            quiet: false,
        };
        let outcome = run_selfplay(&args).unwrap();
        assert!(
            !outcome.ok,
            "200 tick 内不可能决出，门必须失败：{:?}",
            outcome.json
        );
        assert_eq!(outcome.json["decided_at"], serde_json::Value::Null);
    }

    #[test]
    fn bad_numeric_flag_is_usage_error() {
        assert!(matches!(
            parse(&argv("sim-cli scenario --seed abc")),
            Err(CliError::Usage(_))
        ));
    }

    #[test]
    fn scenario_is_deterministic_and_reports_pass() {
        let args = ScenarioArgs {
            seed: 42,
            map: MapSize::Small,
            ticks: 120,
            enable_ai: true,
            expect_hash: None,
            record: None,
            verify_replay: false,
            require_decided: false,
            repeat: 2,
            json: true,
            quiet: false,
        };
        let outcome = run_scenario(&args).unwrap();
        assert!(outcome.ok, "同参重复运行必须稳定：{:?}", outcome.json);
        assert_eq!(outcome.json["determinism"]["stable"], json!(true));
    }

    #[test]
    fn golden_hash_assertion_detects_mismatch() {
        let base = ScenarioArgs {
            seed: 42,
            map: MapSize::Small,
            ticks: 60,
            enable_ai: true,
            expect_hash: Some(0),
            record: None,
            verify_replay: false,
            require_decided: false,
            repeat: 1,
            json: true,
            quiet: false,
        };
        let outcome = run_scenario(&base).unwrap();
        assert!(!outcome.ok, "错误的黄金哈希必须判 FAIL");
        assert_eq!(outcome.json["golden_hash"]["matched"], json!(false));
    }

    #[test]
    fn record_then_replay_roundtrip_matches() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("sim-cli-test-{}.ron", std::process::id()));
        let args = ScenarioArgs {
            seed: 7,
            map: MapSize::Small,
            ticks: 200,
            enable_ai: true,
            expect_hash: None,
            record: Some(path.clone()),
            verify_replay: true,
            require_decided: false,
            repeat: 1,
            json: true,
            quiet: false,
        };
        let outcome = run_scenario(&args).unwrap();
        assert!(outcome.ok, "录制→重放必须一致：{:?}", outcome.json);
        let replay = outcome.json["replay"].clone();
        assert!(
            replay["checkpoints_compared"].as_u64().unwrap() > 0,
            "必须真的比对了检查点：{replay}"
        );
        assert_eq!(replay["mismatches"], json!(0));

        // 再走一次独立的 replay 子命令，确保回放文件可被加载与重放
        let replay_args = ReplayArgs {
            file: path.clone(),
            enable_ai: true,
            expect_final_hash: outcome.json["final_hash"].as_u64(),
            json: true,
        };
        let replay_outcome = run_replay(&replay_args).unwrap();
        assert!(
            replay_outcome.ok,
            "独立 replay 必须通过：{:?}",
            replay_outcome.json
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn v1_replay_fails_fast_on_version() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("sim-cli-v1-{}.ron", std::process::id()));
        fs::write(
            &path,
            "(format_version:1,seed:1,map_size:Small,total_ticks:0,commands_per_tick:{})",
        )
        .unwrap();
        let args = ReplayArgs {
            file: path.clone(),
            enable_ai: true,
            expect_final_hash: None,
            json: true,
        };
        match run_replay(&args) {
            Err(CliError::Format(msg)) => assert!(
                msg.contains("version mismatch"),
                "必须因版本不匹配快速失败，实际：{msg}"
            ),
            other => panic!("v1 回放必须报格式错误，实际：{other:?}"),
        }
        let _ = fs::remove_file(path);
    }
}
