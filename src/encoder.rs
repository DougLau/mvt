// encoder.rs
//
// Copyright (c) 2019 Minnesota Department of Transportation
//
//! Encoder for Mapbox Vector Tile (MVT) geometry.
//!
use std::vec::Vec;

use crate::geom::{Transform,Vec2};

#[derive(Clone,Debug)]
enum Command {
    MoveTo = 1,
    LineTo = 2,
    ClosePath = 7,
}

struct CommandInt {
    id: Command,
    count: u32,
}

struct ParamInt {
    value: i32,
}

/// Geometry types for [Features](struct.Feature.html).
#[derive(Clone,Debug)]
pub enum GeomType {
    /// Point or multipoint
    Point,
    /// Linestring or Multilinestring
    Linestring,
    /// Polygon or Multipolygon
    Polygon,
}

/// Encoder for Point, Linestring or Polygon geometry.
pub struct GeomEncoder {
    geom_tp: GeomType,
    transform: Transform,
    x: i32,
    y: i32,
    cmd_offset: usize,
    count: u32,
    data: Vec<u32>,
}

impl CommandInt {
    fn new(id: Command, count: u32) -> Self {
        CommandInt { id, count }
    }
    fn encode(&self) -> u32 {
        ((self.id.clone() as u32) & 0x7) | (self.count << 3)
    }
}

impl ParamInt {
    fn new(value: i32) -> Self {
        ParamInt { value }
    }
    fn encode(&self) -> u32 {
        ((self.value << 1) ^ (self.value >> 31)) as u32
    }
}

impl GeomEncoder {
    /// Create a new geometry encoder.
    ///
    /// * `geom_tp` Geometry type.
    /// * `transform` Transform to apply to geometry.
    pub fn new(geom_tp: GeomType, transform: Transform) -> Self {
        GeomEncoder {
            geom_tp,
            transform,
            x: 0,
            y: 0,
            count: 0,
            cmd_offset: 0,
            data: vec!(),
        }
    }

    /// Get the geometry type
    pub(crate) fn geom_type(&self) -> GeomType {
        self.geom_tp.clone()
    }

    /// Add a Command
    fn command(&mut self, cmd: Command, count: u32) {
        self.cmd_offset = self.data.len();
        debug!("command: {:?}", &cmd);
        self.data.push(CommandInt::new(cmd, count).encode());
    }

    /// Set count of the most recent Command.
    fn set_command(&mut self, cmd: Command, count: u32) {
        let off = self.cmd_offset;
        self.data[off] = CommandInt::new(cmd, count).encode();
    }

    /// Encode one point with relative coörindates.
    fn encode_point(&mut self, x: f64, y: f64) {
        let p = self.transform * Vec2::new(x, y);
        let x = p.x as i32;
        let y = p.y as i32;
        self.data.push(ParamInt::new(x.saturating_sub(self.x)).encode());
        self.data.push(ParamInt::new(y.saturating_sub(self.y)).encode());
        debug!("point: {},{}", x, y);
        self.x = x;
        self.y = y;
    }

    /// Add a point.
    pub fn add_point(&mut self, x: f64, y: f64) {
        match self.geom_tp {
            GeomType::Point => {
                if self.count == 0 {
                    self.command(Command::MoveTo, 1);
                }
            },
            GeomType::Linestring => {
                match self.count {
                    0 => self.command(Command::MoveTo, 1),
                    1 => self.command(Command::LineTo, 1),
                    _ => (),
                }
            },
            GeomType::Polygon => {
                match self.count {
                    0 => self.command(Command::MoveTo, 1),
                    1 => self.command(Command::LineTo, 1),
                    _ => (),
                }
            },
        }
        self.encode_point(x, y);
        self.count += 1;
    }

    /// Complete the current geometry (for multilinestring / multipolygon).
    pub fn complete_geom(&mut self) {
        match self.geom_tp {
            GeomType::Point => (),
            GeomType::Linestring => {
                if self.count > 1 {
                    self.set_command(Command::LineTo, self.count - 1);
                }
                self.count = 0;
            },
            GeomType::Polygon => {
                if self.count > 1 {
                    self.set_command(Command::LineTo, self.count - 1);
                    self.command(Command::ClosePath, 1);
                }
                self.count = 0;
            },
        }
    }

    /// Get the encoded data.
    pub(crate) fn to_vec(mut self) -> Vec<u32> {
        if let GeomType::Point = self.geom_tp {
            if self.count > 1 {
                self.set_command(Command::MoveTo, self.count);
            }
        } else {
            self.complete_geom();
        }
        self.data
    }
}

#[cfg(test)]
mod test {
    use super::*;
    // Examples from MVT spec:
    #[test]
    fn test_point() {
        let mut pe = GeomEncoder::new(GeomType::Point, Transform::new());
        pe.add_point(25.0, 17.0);
        let b = pe.to_vec();
        assert_eq!(b, vec!(9, 50, 34));
    }
    #[test]
    fn test_multipoint() {
        let mut pe = GeomEncoder::new(GeomType::Point, Transform::new());
        pe.add_point(5.0, 7.0);
        pe.add_point(3.0, 2.0);
        let b = pe.to_vec();
        assert_eq!(b, vec!(17, 10, 14, 3, 9));
    }
    #[test]
    fn test_linestring() {
        let mut le = GeomEncoder::new(GeomType::Linestring, Transform::new());
        le.add_point(2.0, 2.0);
        le.add_point(2.0, 10.0);
        le.add_point(10.0, 10.0);
        let b = le.to_vec();
        assert_eq!(b, vec!(9, 4, 4, 18, 0, 16, 16, 0));
    }
    #[test]
    fn test_multilinestring() {
        let mut le = GeomEncoder::new(GeomType::Linestring, Transform::new());
        le.add_point(2.0, 2.0);
        le.add_point(2.0, 10.0);
        le.add_point(10.0, 10.0);
        le.complete_geom();
        le.add_point(1.0, 1.0);
        le.add_point(3.0, 5.0);
        let b = le.to_vec();
        assert_eq!(b, vec!(9, 4, 4, 18, 0, 16, 16, 0, 9, 17, 17, 10, 4, 8));
    }
    #[test]
    fn test_polygon() {
        let mut pe = GeomEncoder::new(GeomType::Polygon, Transform::new());
        pe.add_point(3.0, 6.0);
        pe.add_point(8.0, 12.0);
        pe.add_point(20.0, 34.0);
        let b = pe.to_vec();
        assert_eq!(b, vec!(9, 6, 12, 18, 10, 12, 24, 44, 15));
    }
    #[test]
    fn test_multipolygon() {
        let mut pe = GeomEncoder::new(GeomType::Polygon, Transform::new());
        pe.add_point(0.0, 0.0);
        pe.add_point(10.0, 0.0);
        pe.add_point(10.0, 10.0);
        pe.add_point(0.0, 10.0);
        pe.complete_geom(); // positive area => exterior ring
        pe.add_point(11.0, 11.0);
        pe.add_point(20.0, 11.0);
        pe.add_point(20.0, 20.0);
        pe.add_point(11.0, 20.0);
        pe.complete_geom(); // positive area => exterior ring
        pe.add_point(13.0, 13.0);
        pe.add_point(13.0, 17.0);
        pe.add_point(17.0, 17.0);
        pe.add_point(17.0, 13.0);
        let b = pe.to_vec(); // negative area => interior ring
        assert_eq!(b, vec!(9, 0, 0, 26, 20, 0, 0, 20, 19, 0, 15, 9, 22, 2, 26,
                           18, 0, 0, 18, 17, 0, 15, 9, 4, 13, 26, 0, 8, 8, 0, 0,
                           7, 15));
    }
}
