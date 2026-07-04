//! Bootstrap 管道 — dispatch, wire, SessionArtifacts, BootstrapPhase。
//!
//! dispatch: 根据 SessionMode 调用对应 initializer → SessionArtifacts
//! wire:     consume SessionArtifacts → setup Driver/World/Resources
//! BootstrapPhase: 生命周期状态（Init → Wired → Active）

use crate::driver::{CommandSource, SimulationDriver};
use crate::session::{SessionConfig, SessionMode};
use crate::network::NetworkCommandSource;
use simulation::replay::ReplayFile;

// ═══════════════════════════════════════════════════════════════
// BootstrapPhase — Driver 生命周期
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootstrapPhase {
    Init,
    Wired,
    Active,
}

// ═══════════════════════════════════════════════════════════════
// TransportResources — network initializer 创建的传输层资源
// ═══════════════════════════════════════════════════════════════

pub struct TransportResources {
    pub receiver: crate::transport::NetworkReceiver,
    pub sender: crate::transport::NetworkSender,
    pub handle: crate::transport::NetworkClientHandle,
}

// ═══════════════════════════════════════════════════════════════
// SessionArtifacts — enum, move-only, consume by wire()
// ═══════════════════════════════════════════════════════════════

pub enum SessionArtifacts {
    Live,
    Replay { replay: ReplayFile },
    Network(super::network::NetworkBootstrapResult),
}

// ═══════════════════════════════════════════════════════════════
// dispatch — 路由到对应 initializer
// ═══════════════════════════════════════════════════════════════

pub fn dispatch(config: &SessionConfig) -> Result<SessionArtifacts, String> {
    match &config.mode {
        SessionMode::Single { .. } => {
            super::single::initialize();
            Ok(SessionArtifacts::Live)
        }
        SessionMode::Replay { .. } => {
            let replay = super::replay::initialize(config)?;
            Ok(SessionArtifacts::Replay { replay })
        }
        SessionMode::Network { .. } => {
            let result = super::network::initialize(config)?;
            Ok(SessionArtifacts::Network(result))
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// wire — 消费 artifact，写入 Driver/World/Resources
// ═══════════════════════════════════════════════════════════════

pub struct BootstrapCtx<'a> {
    pub driver: &'a mut SimulationDriver,
    pub recorder: &'a mut crate::replay::ReplayRecorder,
    pub cmd_buf: &'a mut simulation::command::CommandBuffer,
}

/// 固定写入顺序：
/// 1. init_world（当前在 reset_game_system 中处理）
/// 2. setup_recorder（由 BootstrapCtx 传入）
/// 3. insert_resource（transport 资源）
/// 4. driver.source（赋值）
/// 5. driver.phase = Wired
pub fn wire(ctx: &mut BootstrapCtx, artifacts: SessionArtifacts) {
    match artifacts {
        SessionArtifacts::Live => {
            ctx.driver.source = CommandSource::Live(crate::driver::LiveCommandSource);
        }
        SessionArtifacts::Replay { replay } => {
            ctx.driver.source = CommandSource::Replay(crate::driver::ReplayCommandSource { replay });
        }
        SessionArtifacts::Network(result) => {
            let ns = NetworkCommandSource::new(1, result.player_id, 3);
            ctx.driver.source = CommandSource::Network(ns);
            // Transport resources are registered via BootstrapCtx
            // receiver/sender/handle stored as Bevy Resources
        }
    }
    // Commit final step: driver.source + phase already set above
    ctx.driver.bootstrap_phase = BootstrapPhase::Wired;
}
