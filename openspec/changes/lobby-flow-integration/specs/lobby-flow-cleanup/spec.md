## ADDED Requirements

### Requirement: System ordering

`update_lobby_status` SHALL execute after `lobby_update_system` to ensure state updates are visible.

### Requirement: Cancel cleanup

Cancel button SHALL clean up transport resources and depend on OnExit(Lobby) for lobby-scoped resources.
