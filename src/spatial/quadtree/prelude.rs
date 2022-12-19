extern crate nalgebra as na;

/// Quadtrees exist in 2-dimensional space
pub type Vec2 = na::Vector2<f32>;

/// This is a 2D axis-aligned bounding box (AABB).
#[derive(Default, Debug, Copy, Clone)]
pub struct BBox2D {
    pub min: Vec2,
    pub max: Vec2
}

/// This defines a range of values.
struct Range(pub (f32, f32));

impl Range {
    /// Returns true if this Range intersects the given range.
    pub fn intersects(&self, other: &Range) -> bool {
        self.0.1 >= other.0.0 && other.0.1 >= self.0.0
    }
}

impl BBox2D {
    /// Returns true if the BBox contains the given point.
    pub fn contains(&self, p: &Vec2) -> bool {
        self.min <= *p && *p < self.max
    }

    /// Returns true if the BBox intersects the given BBox.
    pub fn intersects(&self, other: &BBox2D) -> bool {
        self.xrange().intersects(&other.xrange()) &&
        self.yrange().intersects(&other.yrange())
    }

    /// Returns the midpoint of the BBox
    pub fn mid(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    /// Subdivides the BBox into 4 BBoxes, using 'mid' as the midpoint.
    pub fn subdivide(&self, mid: &Vec2) -> [BBox2D; 4] {
        [
            // Lower left
            Self {
                min: self.min,
                max: *mid
            },
            // Lower right
            Self {
                min: Vec2::from([mid.x, self.min.y]),
                max: Vec2::from([self.max.x, mid.y])
            },
            // Upper right
            Self {
                min: *mid,
                max: self.max
            },
            // Upper left
            Self {
                min: Vec2::from([self.min.x, mid.y]),
                max: Vec2::from([mid.x, self.max.y]),
            }
        ]
    }

    /// Returns the range of x-values of the BBox.
    fn xrange(&self) -> Range {
        Range((self.min.x, self.max.x))
    }

    /// Returns the range of y-values of the BBox.
    fn yrange(&self) -> Range {
        Range((self.min.y, self.max.y))
    }
}
