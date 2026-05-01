//! Dirty region tracking for efficient partial screen updates.
//!
//! Tracks which screen regions have changed and need redrawing,
//! enabling targeted rendering instead of fullscreen refreshes.

/// A rectangular dirty region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyRegion {
    /// The x coordinate of this region.
    pub x: u16,
    /// The y coordinate of this region.
    pub y: u16,
    /// The width of this region.
    pub width: u16,
    /// The height of this region.
    pub height: u16,
}

impl DirtyRegion {
    /// Creates a new dirty region.
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns true if this region intersects with another.
    pub fn intersects(&self, other: &DirtyRegion) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Returns the intersection of two regions, if any.
    pub fn intersection(&self, other: &DirtyRegion) -> Option<DirtyRegion> {
        if !self.intersects(other) {
            return None;
        }
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let width = (self.x + self.width).min(other.x + other.width) - x;
        let height = (self.y + self.height).min(other.y + other.height) - y;
        Some(DirtyRegion::new(x, y, width, height))
    }

    /// Expands this region to include a point.
    pub fn expand(&mut self, x: u16, y: u16) {
        let x2 = x.max(self.x + self.width);
        let y2 = y.max(self.y + self.height);
        self.x = self.x.min(x);
        self.y = self.y.min(y);
        self.width = x2 - self.x;
        self.height = y2 - self.y;
    }
}

/// Tracks dirty regions for efficient rendering.
pub struct DirtyRegionTracker {
    regions: Vec<DirtyRegion>,
    full_refresh: bool,
}

impl DirtyRegionTracker {
    /// Creates a new dirty region tracker.
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            full_refresh: true,
        }
    }

    /// Marks the entire screen as dirty, requiring full refresh.
    pub fn mark_all_dirty(&mut self) {
        self.full_refresh = true;
        self.regions.clear();
    }

    /// Marks a rectangular region as dirty.
    pub fn mark_dirty(&mut self, x: u16, y: u16, width: u16, height: u16) {
        self.full_refresh = false;
        let region = DirtyRegion::new(x, y, width, height);

        for existing in &mut self.regions {
            if let Some(intersection) = existing.intersection(&region) {
                existing.expand(intersection.x, intersection.y);
                existing.expand(
                    intersection.x + intersection.width,
                    intersection.y + intersection.height,
                );
                return;
            }
        }

        self.regions.push(region);
    }

    /// Marks a cell as dirty.
    pub fn mark_cell_dirty(&mut self, x: u16, y: u16) {
        self.mark_dirty(x, y, 1, 1);
    }

    /// Returns all dirty regions that need redrawing.
    pub fn dirty_regions(&self) -> &[DirtyRegion] {
        &self.regions
    }

    /// Returns true if a full refresh is needed.
    pub fn needs_full_refresh(&self) -> bool {
        self.full_refresh
    }

    /// Returns true if there are any dirty regions.
    pub fn is_dirty(&self) -> bool {
        self.full_refresh || !self.regions.is_empty()
    }

    /// Clears all dirty regions.
    pub fn clear(&mut self) {
        self.regions.clear();
        self.full_refresh = false;
    }
}

impl Default for DirtyRegionTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_region_intersects() {
        let r1 = DirtyRegion::new(0, 0, 10, 10);
        let r2 = DirtyRegion::new(5, 5, 10, 10);
        assert!(r1.intersects(&r2));
    }

    #[test]
    fn test_dirty_region_no_intersect() {
        let r1 = DirtyRegion::new(0, 0, 10, 10);
        let r2 = DirtyRegion::new(20, 20, 10, 10);
        assert!(!r1.intersects(&r2));
    }

    #[test]
    fn test_dirty_region_expand() {
        let mut r = DirtyRegion::new(5, 5, 5, 5);
        r.expand(3, 3);
        assert_eq!(r.x, 3);
        assert_eq!(r.y, 3);
        assert_eq!(r.width, 7);
        assert_eq!(r.height, 7);
    }

    #[test]
    fn test_tracker_mark_dirty() {
        let mut tracker = DirtyRegionTracker::new();
        tracker.mark_dirty(10, 10, 20, 20);
        assert!(tracker.is_dirty());
        assert!(!tracker.needs_full_refresh());
    }

    #[test]
    fn test_tracker_full_refresh() {
        let mut tracker = DirtyRegionTracker::new();
        tracker.mark_all_dirty();
        assert!(tracker.needs_full_refresh());
    }

    #[test]
    fn test_tracker_clear() {
        let mut tracker = DirtyRegionTracker::new();
        tracker.mark_dirty(10, 10, 20, 20);
        tracker.clear();
        assert!(!tracker.is_dirty());
    }
}
