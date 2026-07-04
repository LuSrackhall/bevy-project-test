//! Session Bootstrap Layer — 标准化初始化管道。
//!
//! 将 UI 意图（GameIntent）转换为 Driver 就绪态。
//! 详见 openspec/changes/session-bootstrap-layer/brainstorm-spec.md

pub mod bootstrap;
pub mod network;
pub mod replay;
pub mod single;

use simulation::map::MapSize;
use std::path::PathBuf;

// ═══════════════════════════════════════════════════════════════
// SessionConfig — 初始化配置，由 resolve_intent() 从 GameIntent 转换
// ═══════════════════════════════════════════════════════════════

/// Session 配置。生命周期：bootstrap-scoped，bootstrap 完成后必须释放。
pub struct SessionConfig {
    pub mode: SessionMode,
}

pub enum SessionMode {
    Single { map_size: MapSize },
    Replay { path: PathBuf },
    Network { relay_addr: String, player_count: u8 },
}
