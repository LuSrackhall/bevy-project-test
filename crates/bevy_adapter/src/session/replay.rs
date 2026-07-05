//! Replay mode initializer — 加载 ReplayFile。

use crate::session::SessionConfig;
use simulation::replay::ReplayFile;

pub fn initialize(config: &SessionConfig) -> Result<ReplayFile, String> {
    let path = match &config.mode {
        crate::session::SessionMode::Replay { path } => path,
        _ => return Err("Not a Replay session".into()),
    };
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read replay file '{}': {}", path.display(), e))?;
    ReplayFile::from_ron(&content)
        .map_err(|e| format!("Invalid replay file '{}': {}", path.display(), e))
}
