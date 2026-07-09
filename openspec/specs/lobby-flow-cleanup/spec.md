# lobby-flow-cleanup Specification

## Purpose
TBD - created by archiving change lobby-flow-integration. Update Purpose after archive.
## Requirements
### Requirement: System ordering

`update_lobby_status` SHALL execute after `lobby_update_system` to ensure state updates are visible.

### Requirement: Cancel cleanup

Cancel button SHALL clean up transport resources and depend on OnExit(Lobby) for lobby-scoped resources.

