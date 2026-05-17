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
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);
        let width = right.saturating_sub(x);
        let height = bottom.saturating_sub(y);
        if width == 0 || height == 0 {
            return None;
        }
        Some(DirtyRegion::new(x, y, width, height))
    }

    /// Expands this region to include a point.
    pub fn expand(&mut self, x: u16, y: u16) {
        let x2 = x.max(self.x.saturating_add(self.width));
        let y2 = y.max(self.y.saturating_add(self.height));
        self.x = self.x.min(x);
        self.y = self.y.min(y);
        self.width = x2.saturating_sub(self.x);
        self.height = y2.saturating_sub(self.y);
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

        let mut merged = false;
        for existing in &mut self.regions {
            if existing.intersects(&region) || self.adjacent(existing, &region) {
                existing.expand(region.x, region.y);
                existing.expand(
                    region.x.saturating_add(region.width),
                    region.y.saturating_add(region.height),
                );
                merged = true;
                break;
            }
        }

        if !merged {
            self.regions.push(region);
        } else {
            self.merge_pass();
        }
    }

    fn adjacent(&self, a: &DirtyRegion, b: &DirtyRegion) -> bool {
        let a_right = a.x.saturating_add(a.width);
        let a_bottom = a.y.saturating_add(a.height);
        let b_right = b.x.saturating_add(b.width);
        let b_bottom = b.y.saturating_add(b.height);
        a.x <= b_right && b.x <= a_right && a.y <= b_bottom && b.y <= a_bottom
    }

    fn merge_pass(&mut self) {
        if self.regions.len() <= 1 {
            return;
        }
        let mut changed = true;
        while changed {
            changed = false;
            let mut i = 0;
            while i < self.regions.len() {
                let mut j = i + 1;
                while j < self.regions.len() {
                    if self.regions[i].intersects(&self.regions[j])
                        || self.adjacent(&self.regions[i], &self.regions[j])
                    {
                        let other = self.regions.swap_remove(j);
                        self.regions[i].expand(other.x, other.y);
                        self.regions[i].expand(
                            other.x.saturating_add(other.width),
                            other.y.saturating_add(other.height),
                        );
                        changed = true;
                    } else {
                        j += 1;
                    }
                }
                i += 1;
            }
        }
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
