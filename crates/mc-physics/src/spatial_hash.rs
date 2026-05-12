//! Spatial hash grid for broad-phase spatial queries.
//!
//! Partitions 3D space into a uniform grid of cells, enabling efficient
//! radius and AABB queries without checking every item.

use std::collections::HashMap;

/// A spatial hash grid that partitions 3D space into uniform cells.
#[derive(Debug, Clone)]
pub struct SpatialHash<T: Clone> {
    cells: HashMap<(i32, i32, i32), Vec<T>>,
    cell_size: f32,
}

impl<T: Clone> SpatialHash<T> {
    /// Creates a new spatial hash with the given cell size.
    pub fn new(cell_size: f32) -> Self {
        Self {
            cells: HashMap::new(),
            cell_size,
        }
    }

    /// Returns the cell key for a world-space position.
    fn cell_key(&self, pos: [f32; 3]) -> (i32, i32, i32) {
        let inv = 1.0 / self.cell_size;
        (
            (pos[0] * inv).floor() as i32,
            (pos[1] * inv).floor() as i32,
            (pos[2] * inv).floor() as i32,
        )
    }

    /// Inserts an item at the given position.
    pub fn insert(&mut self, pos: [f32; 3], item: T) {
        let key = self.cell_key(pos);
        self.cells.entry(key).or_default().push(item);
    }

    /// Returns references to all items within `radius` of `center`.
    ///
    /// This is a broad-phase query that returns all items in cells that
    /// overlap the bounding sphere. Callers should perform fine-grained
    /// distance checks if exact radius filtering is needed.
    pub fn query_radius(&self, center: [f32; 3], radius: f32) -> Vec<&T> {
        let min = [
            center[0] - radius,
            center[1] - radius,
            center[2] - radius,
        ];
        let max = [
            center[0] + radius,
            center[1] + radius,
            center[2] + radius,
        ];
        self.query_aabb(min, max)
    }

    /// Returns references to all items in cells overlapping the given AABB.
    pub fn query_aabb(&self, min: [f32; 3], max: [f32; 3]) -> Vec<&T> {
        let min_key = self.cell_key(min);
        let max_key = self.cell_key(max);

        let mut results = Vec::new();
        for x in min_key.0..=max_key.0 {
            for y in min_key.1..=max_key.1 {
                for z in min_key.2..=max_key.2 {
                    if let Some(items) = self.cells.get(&(x, y, z)) {
                        results.extend(items.iter());
                    }
                }
            }
        }
        results
    }

    /// Removes all items from the grid.
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Returns the number of occupied cells.
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_is_empty() {
        let grid: SpatialHash<u32> = SpatialHash::new(16.0);
        assert_eq!(grid.cell_count(), 0);
    }

    #[test]
    fn insert_and_query_single_item() {
        let mut grid = SpatialHash::new(16.0);
        grid.insert([5.0, 5.0, 5.0], 42);
        let results = grid.query_radius([5.0, 5.0, 5.0], 1.0);
        assert_eq!(results.len(), 1);
        assert_eq!(*results[0], 42);
    }

    #[test]
    fn query_radius_finds_nearby_items() {
        let mut grid = SpatialHash::new(16.0);
        grid.insert([0.0, 0.0, 0.0], "a");
        grid.insert([1.0, 0.0, 0.0], "b");
        grid.insert([100.0, 100.0, 100.0], "far");

        let results = grid.query_radius([0.5, 0.0, 0.0], 2.0);
        assert!(results.contains(&&"a"));
        assert!(results.contains(&&"b"));
        assert!(!results.contains(&&"far"));
    }

    #[test]
    fn query_aabb_returns_items_in_range() {
        let mut grid = SpatialHash::new(10.0);
        grid.insert([5.0, 5.0, 5.0], 1);
        grid.insert([15.0, 15.0, 15.0], 2);
        grid.insert([50.0, 50.0, 50.0], 3);

        let results = grid.query_aabb([0.0, 0.0, 0.0], [20.0, 20.0, 20.0]);
        assert!(results.contains(&&1));
        assert!(results.contains(&&2));
        assert!(!results.contains(&&3));
    }

    #[test]
    fn clear_removes_all_items() {
        let mut grid = SpatialHash::new(16.0);
        grid.insert([0.0, 0.0, 0.0], 1);
        grid.insert([10.0, 10.0, 10.0], 2);
        assert!(grid.cell_count() > 0);

        grid.clear();
        assert_eq!(grid.cell_count(), 0);
        assert!(grid.query_radius([0.0, 0.0, 0.0], 100.0).is_empty());
    }

    #[test]
    fn cell_count_tracks_occupied_cells() {
        let mut grid = SpatialHash::new(10.0);
        grid.insert([0.0, 0.0, 0.0], "a");
        grid.insert([1.0, 1.0, 1.0], "b"); // same cell
        assert_eq!(grid.cell_count(), 1);

        grid.insert([20.0, 20.0, 20.0], "c"); // different cell
        assert_eq!(grid.cell_count(), 2);
    }

    #[test]
    fn negative_coordinates_work() {
        let mut grid = SpatialHash::new(16.0);
        grid.insert([-5.0, -5.0, -5.0], "neg");
        let results = grid.query_radius([-5.0, -5.0, -5.0], 1.0);
        assert_eq!(results.len(), 1);
        assert_eq!(*results[0], "neg");
    }

    #[test]
    fn multiple_items_same_cell() {
        let mut grid = SpatialHash::new(16.0);
        grid.insert([1.0, 1.0, 1.0], 10);
        grid.insert([2.0, 2.0, 2.0], 20);
        grid.insert([3.0, 3.0, 3.0], 30);

        let results = grid.query_radius([2.0, 2.0, 2.0], 0.5);
        assert_eq!(results.len(), 3);
    }
}
