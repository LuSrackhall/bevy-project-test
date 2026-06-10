//! Spatial hash grid for O(1) neighbor queries.

use std::collections::HashMap;
use crate::types::*;

/// A spatial hash grid storing (position, collision_radius) pairs.
pub struct SpatialHash {
    cell_size: i64,
    cells: HashMap<(i32, i32), Vec<(FixedVec2, u32)>>,
}

impl SpatialHash {
    pub fn new(cell_size: Fixed) -> Self {
        SpatialHash { cell_size: cell_size.0, cells: HashMap::new() }
    }

    /// Insert a position with its collision radius.
    pub fn insert(&mut self, pos: FixedVec2, radius: u32) {
        let key = self.cell_key(pos);
        self.cells.entry(key).or_default().push((pos, radius));
    }

    /// Return all (position, radius) entries in the 9 cells around center.
    /// Caller filters by distance using per-unit radii.
    pub fn query_nearby(&self, center: FixedVec2) -> Vec<(FixedVec2, u32)> {
        let (cx, cy) = self.cell_key(center);
        let mut result = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(cell) = self.cells.get(&(cx + dx, cy + dy)) {
                    result.extend(cell);
                }
            }
        }
        result
    }

    fn cell_key(&self, pos: FixedVec2) -> (i32, i32) {
        let x = (pos.x.0 / self.cell_size) as i32;
        let y = (pos.y.0 / self.cell_size) as i32;
        (x, y)
    }
}
