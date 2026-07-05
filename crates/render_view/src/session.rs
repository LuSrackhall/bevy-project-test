//! UI 层类型：GameIntent + resolve_intent()。
//!
//! GameIntent 属于 render_view（UI 层类型）。
//! resolve_intent() 是纯转换函数（render_view 已依赖 bevy_adapter）。

use simulation::map::MapSize;
use std::path::PathBuf;

/// UI 发出的启动意图。属于 UI 层类型。
pub struct GameIntent {
    pub kind: GameKind,
}

pub enum GameKind {
    Single { map_size: MapSize },
    Replay { path: PathBuf },
    Network { relay_addr: String, player_count: u8 },
}

/// 纯转换：GameIntent → SessionConfig。
/// render_view 已依赖 bevy_adapter，因此翻译放在此层不会产生反向依赖。
pub fn resolve_intent(intent: GameIntent) -> bevy_adapter::session::SessionConfig {
    use bevy_adapter::session::SessionMode;
    let mode = match intent.kind {
        GameKind::Single { map_size } => SessionMode::Single { map_size },
        GameKind::Replay { path } => SessionMode::Replay { path },
        GameKind::Network { relay_addr, player_count } => {
            SessionMode::Network { relay_addr, player_count, player_id: 0 }
        }
    };
    bevy_adapter::session::SessionConfig { mode }
}
