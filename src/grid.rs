// grid.rs
//
// Copyright (c) 2019 Minnesota Department of Transportation
//
use crate::Error;
use crate::geom::{Transform, Vec2};

#[derive(Clone)]
pub struct BBox {
    top_left: Vec2,
    bottom_right: Vec2,
}

#[derive(Clone)]
pub struct TileId {
    x: u32,
    y: u32,
    z: u32,
}

pub struct Grid {
    srid: i32,
    bbox: BBox,
}

impl BBox {
    pub fn new(top_left: Vec2, bottom_right: Vec2) -> Self {
        BBox { top_left, bottom_right }
    }

    pub fn x_min(&self) -> f64 {
        self.top_left.x.min(self.bottom_right.x)
    }

    pub fn x_max(&self) -> f64 {
        self.top_left.x.max(self.bottom_right.x)
    }

    pub fn y_min(&self) -> f64 {
        self.top_left.y.min(self.bottom_right.y)
    }

    pub fn y_max(&self) -> f64 {
        self.top_left.y.max(self.bottom_right.y)
    }

    fn x_span(&self) -> f64 {
        self.bottom_right.x - self.top_left.x
    }

    fn y_span(&self) -> f64 {
        self.bottom_right.y - self.top_left.y
    }
}

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
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        TileId { x, y, z }
    }

    pub fn check_valid(&self) -> Result<(), Error> {
        if self.z > 31 {
            return Err(Error::InvalidTid());
        }
        let s = 1 << self.z;
        if self.x < s && self.y < s {
            Ok(())
        } else {
            Err(Error::InvalidTid())
        }
    }
}

impl Grid {
    fn new(srid: i32, bbox: BBox) -> Self {
        Grid { srid, bbox }
    }

    pub fn new_web_mercator() -> Self {
        const HALF_SIZE_M: f64 = 20037508.3427892480;
        let srid = 3857;
        let top_left = Vec2::new(-HALF_SIZE_M, HALF_SIZE_M);
        let bottom_right = Vec2::new(HALF_SIZE_M, -HALF_SIZE_M);
        let bbox = BBox::new(top_left, bottom_right);
        Grid::new(srid, bbox)
    }

    pub fn srid(&self) -> i32 {
        self.srid
    }

    pub fn tile_bounds(&self, tid: TileId) -> Result<BBox, Error> {
        tid.check_valid()?;
        let tz = SCALE[tid.z as usize];
        let sx = self.bbox.x_span() * tz;
        let sy = self.bbox.y_span() * tz;
        let tx = self.bbox.top_left.x;
        let ty = self.bbox.top_left.y;
        let t = Transform::new_scale(sx, sy).translate(tx, ty);
        let tidx = tid.x as f64;
        let tidy = tid.y as f64;
        let top_left = t * Vec2::new(tidx, tidy);
        let bottom_right = t * Vec2::new(tidx + 1.0, tidy + 1.0);
        Ok(BBox::new(top_left, bottom_right))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_tile_bounds() {
        let g = Grid::new_web_mercator();
        if let Ok(b) = g.tile_bounds(TileId::new(0, 0, 0)) {
            assert_eq!(b.top_left,
                       Vec2::new(-20037508.3427892480, 20037508.3427892480));
            assert_eq!(b.bottom_right,
                       Vec2::new(20037508.3427892480, -20037508.3427892480));
        } else {
            assert!(false);
        }
        if let Ok(b) = g.tile_bounds(TileId::new(0, 0, 1)) {
            assert_eq!(b.top_left,
                       Vec2::new(-20037508.3427892480, 20037508.3427892480));
            assert_eq!(b.bottom_right,
                       Vec2::new(0.0, 0.0));
        } else {
            assert!(false);
        }
        if let Ok(b) = g.tile_bounds(TileId::new(1, 1, 1)) {
            assert_eq!(b.top_left,
                       Vec2::new(0.0, 0.0));
            assert_eq!(b.bottom_right,
                       Vec2::new(20037508.3427892480, -20037508.3427892480));
        } else {
            assert!(false);
        }
        if let Ok(b) = g.tile_bounds(TileId::new(246, 368, 10)) {
            assert_eq!(b.top_left,
                       Vec2::new(-10410111.756214727, 5635549.221409475));
            assert_eq!(b.bottom_right,
                       Vec2::new(-10370975.997732716, 5596413.462927466));
        } else {
            assert!(false);
        }
    }
}
