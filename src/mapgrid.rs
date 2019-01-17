// mapgrid.rs
//
// Copyright (c) 2019 Minnesota Department of Transportation
//
//! BBox, TileId and MapGrid structs.
//!
use crate::Error;
use crate::geom::{Transform, Vec2};

/// A bounding box is an axis-aligned rectangle.  It is defined by two corners:
/// north_west and south_east.
///
/// # Example
/// ```
/// use mvt::{BBox, Vec2};
/// let north_west = Vec2::new(-10.0, 0.0);
/// let south_east = Vec2::new(10.0, 8.0);
/// let bbox = BBox::new(north_west, south_east);
/// ```
#[derive(Clone)]
pub struct BBox {
    north_west: Vec2,
    south_east: Vec2,
}

/// A tile ID identifies a tile on a map grid at a specific zoom level.  It uses
/// XYZ addressing, with X increasing from west to east and Y increasing from
/// north to south.  The X and Y values can range from 0 to 2<sup>Z</sup>-1.
#[derive(Clone)]
pub struct TileId {
    x: u32,
    y: u32,
    z: u32,
}

/// A map grid is used to address [tile](struct.Tile.html)s on a map.
/// The grid should be in projected coördinates.
pub struct MapGrid {
    srid: i32,
    bbox: BBox,
}

impl BBox {
    /// Create a new bounding box.
    ///
    /// * `north_west` The north-west (top-left) corner of the bounds.
    /// * `south_east` The south-east (bottom-right) corner of the bounds.
    pub fn new(north_west: Vec2, south_east: Vec2) -> Self {
        BBox { north_west, south_east }
    }

    /// Get the minimum X value.
    pub fn x_min(&self) -> f64 {
        self.north_west.x.min(self.south_east.x)
    }

    /// Get the maximum X value.
    pub fn x_max(&self) -> f64 {
        self.north_west.x.max(self.south_east.x)
    }

    /// Get the minimum Y value.
    pub fn y_min(&self) -> f64 {
        self.north_west.y.min(self.south_east.y)
    }

    /// Get the maximum Y value.
    pub fn y_max(&self) -> f64 {
        self.north_west.y.max(self.south_east.y)
    }

    /// Get the X span.
    fn x_span(&self) -> f64 {
        self.south_east.x - self.north_west.x
    }

    /// Get the Y span.
    fn y_span(&self) -> f64 {
        self.south_east.y - self.north_west.y
    }
}

/// Scales at each zoom level.
const SCALE: [f64; 32] = [
    // Someday, we can use const fn...
    1.0 / (1 << 0) as f64, 1.0 / (1 << 1) as f64,
    1.0 / (1 << 2) as f64, 1.0 / (1 << 3) as f64,
    1.0 / (1 << 4) as f64, 1.0 / (1 << 5) as f64,
    1.0 / (1 << 6) as f64, 1.0 / (1 << 7) as f64,
    1.0 / (1 << 8) as f64, 1.0 / (1 << 9) as f64,
    1.0 / (1 << 10) as f64, 1.0 / (1 << 11) as f64,
    1.0 / (1 << 12) as f64, 1.0 / (1 << 13) as f64,
    1.0 / (1 << 14) as f64, 1.0 / (1 << 15) as f64,
    1.0 / (1 << 16) as f64, 1.0 / (1 << 17) as f64,
    1.0 / (1 << 18) as f64, 1.0 / (1 << 19) as f64,
    1.0 / (1 << 20) as f64, 1.0 / (1 << 21) as f64,
    1.0 / (1 << 22) as f64, 1.0 / (1 << 23) as f64,
    1.0 / (1 << 24) as f64, 1.0 / (1 << 25) as f64,
    1.0 / (1 << 26) as f64, 1.0 / (1 << 27) as f64,
    1.0 / (1 << 28) as f64, 1.0 / (1 << 29) as f64,
    1.0 / (1 << 30) as f64, 1.0 / (1 << 31) as f64,
];

impl TileId {
    /// Create a new TildId.
    ///
    /// If invalid, returns [Error::InvalidTid](enum.Error.html).
    pub fn new(x: u32, y: u32, z: u32) -> Result<Self, Error> {
        TileId::check_valid(x, y, z)?;
        Ok(TileId { x, y, z })
    }

    /// Check whether a tile ID is valid.
    fn check_valid(x: u32, y: u32, z: u32) -> Result<(), Error> {
        if z > 31 {
            return Err(Error::InvalidTid());
        }
        let s = 1 << z;
        if x < s && y < s {
            Ok(())
        } else {
            Err(Error::InvalidTid())
        }
    }
}

impl MapGrid {
    /// Create a new map grid.
    pub fn new(srid: i32, bbox: BBox) -> Self {
        MapGrid { srid, bbox }
    }

    /// Create a new map grid using web mercator coördinates.
    pub fn new_web_mercator() -> Self {
        const HALF_SIZE_M: f64 = 20037508.3427892480;
        let srid = 3857;
        let north_west = Vec2::new(-HALF_SIZE_M, HALF_SIZE_M);
        let south_east = Vec2::new(HALF_SIZE_M, -HALF_SIZE_M);
        let bbox = BBox::new(north_west, south_east);
        MapGrid::new(srid, bbox)
    }

    /// Get the spatial reference ID.
    pub fn srid(&self) -> i32 {
        self.srid
    }

    /// Get the bounding box of a tile ID.
    pub fn tile_bounds(&self, tid: TileId) -> BBox {
        let tz = SCALE[tid.z as usize];
        let sx = self.bbox.x_span() * tz;
        let sy = self.bbox.y_span() * tz;
        let tx = self.bbox.north_west.x;
        let ty = self.bbox.north_west.y;
        let t = Transform::new_scale(sx, sy).translate(tx, ty);
        let tidx = tid.x as f64;
        let tidy = tid.y as f64;
        let north_west = t * Vec2::new(tidx, tidy);
        let south_east = t * Vec2::new(tidx + 1.0, tidy + 1.0);
        BBox::new(north_west, south_east)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_tile_bounds() {
        let g = MapGrid::new_web_mercator();
        if let Ok(tid) = TileId::new(0, 0, 0) {
            let b = g.tile_bounds(tid);
            assert_eq!(b.north_west,
                       Vec2::new(-20037508.3427892480, 20037508.3427892480));
            assert_eq!(b.south_east,
                       Vec2::new(20037508.3427892480, -20037508.3427892480));
        } else {
            assert!(false);
        }
        if let Ok(tid) = TileId::new(0, 0, 1) {
            let b = g.tile_bounds(tid);
            assert_eq!(b.north_west,
                       Vec2::new(-20037508.3427892480, 20037508.3427892480));
            assert_eq!(b.south_east,
                       Vec2::new(0.0, 0.0));
        } else {
            assert!(false);
        }
        if let Ok(tid) = TileId::new(1, 1, 1) {
            let b = g.tile_bounds(tid);
            assert_eq!(b.north_west,
                       Vec2::new(0.0, 0.0));
            assert_eq!(b.south_east,
                       Vec2::new(20037508.3427892480, -20037508.3427892480));
        } else {
            assert!(false);
        }
        if let Ok(tid) = TileId::new(246, 368, 10) {
            let b = g.tile_bounds(tid);
            assert_eq!(b.north_west,
                       Vec2::new(-10410111.756214727, 5635549.221409475));
            assert_eq!(b.south_east,
                       Vec2::new(-10370975.997732716, 5596413.462927466));
        } else {
            assert!(false);
        }
    }
}
