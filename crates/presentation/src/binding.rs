// Binding types now live in bevy_adapter::binding to avoid circular deps.
// Re-export for convenience.
pub use bevy_adapter::binding::{LogicEntityRef, PresentationPosition, InterpolationData};
