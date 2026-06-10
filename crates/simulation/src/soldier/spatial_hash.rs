//! Spatial hash grid for O(1) neighbor queries in unit separation.

use std::collections::HashMap;
use crate::types::*;

/// A spatial hash grid that maps positions to entities for fast neighbor lookups.
pub struct SpatialHash {
    cell_size: i64, // in Fixed internal units
    cells: HashMap<(i32, i32), Vec<(FixedVec2)>>,
}

impl SpatialHash {
    /// Create a new spatial hash with the given cell size (in Fixed internal units).
    pub fn new(cell_size: Fixed) -> Self {
        SpatialHash {
            cell_size: cell_size.0,
            cells: HashMap::new(),
        }
    }

    /// Insert a position into the grid.
    pub fn insert(&mut self, pos: FixedVec2) {
        let key = self.cell_key(pos);
        self.cells.entry(key).or_default().push(pos);
    }

    /// Query all positions within `radius` of `center`.
    /// Checks the center cell + 8 adjacent cells (9 total).
    pub fn query_nearby(&self, center: FixedVec2, radius: Fixed) -> Vec<FixedVec2> {
        let radius_sq = (radius * radius).0;
        let (cx, cy) = self.cell_key(center);
        let mut result = Vec::new();

        for dx in -1..=1 {
            for dy in -1..=1 {
                let key = (cx + dx, cy + dy);
                if let Some(cell) = self.cells.get(&key) {
                    for &pos in cell {
                        let ds = (center - pos).length_squared();
                        if ds.0 < radius_sq && ds.0 > 0 {
                            result.push(pos);
                        }
                    }
                }
            }
        }

        result
    }

    /// Compute the cell key for a position.
    fn cell_key(&self, pos: FixedVec2) -> (i32, i32) {
        let x = (pos.x.0 / self.cell_size) as i32;
        let y = (pos.y.0 / self.cell_size) as i32;
        (x, y)
    }
}
