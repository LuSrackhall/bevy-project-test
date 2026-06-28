//! Spatial hash grid for deterministic O(k) neighbor queries.
//!
//! Uses BTreeMap for deterministic cell traversal order and UnitId sorting
//! within cells for deterministic intra-cell iteration (constitution §2.6).

use crate::types::*;
use std::collections::BTreeMap;

/// Entry in a spatial hash cell: position, collision radius, and owning UnitId.
#[derive(Clone, Copy)]
pub struct SpatialEntry {
    pub pos: FixedVec2,
    pub radius: u32,
    pub unit_id: UnitId,
}

/// A spatial hash grid storing SpatialEntry pairs.
/// BTreeMap ensures deterministic cell traversal; cells are sorted by UnitId.
pub struct SpatialHash {
    cell_size: i64,
    cells: BTreeMap<(i32, i32), Vec<SpatialEntry>>,
}

impl SpatialHash {
    pub fn new(cell_size: Fixed) -> Self {
        SpatialHash {
            cell_size: cell_size.0,
            cells: BTreeMap::new(),
        }
    }

    /// Insert an entry. Cell-internal Vec is kept sorted by UnitId for determinism.
    pub fn insert(&mut self, entry: SpatialEntry) {
        let key = self.cell_key(entry.pos);
        let cell = self.cells.entry(key).or_default();
        let pos = cell.partition_point(|e| e.unit_id < entry.unit_id);
        cell.insert(pos, entry);
    }

    /// Return all entries in the 9 cells around center.
    /// Traversal order: BTreeMap cell key order (deterministic), then UnitId-sorted within cell.
    pub fn query_nearby(&self, center: FixedVec2) -> Vec<SpatialEntry> {
        let (cx, cy) = self.cell_key(center);
        let mut result = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(cell) = self.cells.get(&(cx + dx, cy + dy)) {
                    result.extend_from_slice(cell);
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
