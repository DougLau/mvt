// encoder.rs
//
// Copyright (c) 2019-2024  Minnesota Department of Transportation
//
//! Encoder for Mapbox Vector Tile (MVT) geometry.
//!
use crate::error::{Error, Result};
use pointy::{BBox, Float, Transform};

/// Path commands
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Command {
    /// Move to new position
    MoveTo = 1,

    /// Line to new position
    LineTo = 2,

    /// Close current path
    ClosePath = 7,
}

/// Integer command
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct CommandInt {
    /// Path command
    id: Command,

    /// Command count
    count: u32,
}

/// Integer parameter
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct ParamInt {
    /// Parameter value
    value: i32,
}

/// Geometry types for [Features](struct.Feature.html).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GeomType {
    /// Point or Multipoint
    #[default]
    Point,

    /// Linestring or Multilinestring
    Linestring,

    /// Polygon or Multipolygon
    Polygon,
}

/// Encoder for [Feature](struct.Feature.html) geometry.
///
/// This can consist of Point, Linestring or Polygon data.
///
/// # Example
/// ```
/// # use mvt::{Error, GeomEncoder, GeomType};
/// # use pointy::Transform;
/// # fn main() -> Result<(), Error> {
/// let geom_data = GeomEncoder::new(GeomType::Point)
///     .point(0.0, 0.0)?
///     .point(10.0, 0.0)?
///     .encode()?;
/// # Ok(()) }
/// ```
#[derive(Default)]
pub struct GeomEncoder<F>
where
    F: Float,
{
    /// Geometry type
    geom_tp: GeomType,

    /// Bounding box
    bbox: BBox<F>,

    /// Minimum X value
    x_min: i32,

    /// Maximum X value
    x_max: i32,

    /// Minimum Y value
    y_min: i32,

    /// Maximum Y value
    y_max: i32,

    /// Transform to MVT coordinates
    transform: Transform<F>,

    /// Current point
    pt: Option<(i32, i32)>,

    /// Previous point
    prev_pt: Option<(i32, i32)>,

    /// Command offset
    cmd_offset: usize,

    /// Count of geometry data
    count: u32,

    /// Encoded geometry data
    data: Vec<u32>,
}

/// Validated geometry data for [Feature](struct.Feature.html)s.
///
/// Use [GeomEncoder](struct.GeomEncoder.html) to encode.
///
/// # Example
/// ```
/// # use mvt::{Error, GeomEncoder, GeomType};
/// # use pointy::Transform;
/// # fn main() -> Result<(), Error> {
/// let geom_data = GeomEncoder::new(GeomType::Point)
///     .point(0.0, 0.0)?
///     .point(10.0, 0.0)?
///     .encode()?;
/// # Ok(()) }
/// ```
pub struct GeomData {
    /// Geometry type
    geom_tp: GeomType,

    /// Encoded geometry data
    data: Vec<u32>,
}

impl CommandInt {
    /// Create a new integer command
    fn new(id: Command, count: u32) -> Self {
        debug_assert!(count <= 0x1FFF_FFFF);
        CommandInt { id, count }
    }

    /// Encode command
    fn encode(&self) -> u32 {
        ((self.id as u32) & 0x7) | (self.count << 3)
    }
}

impl ParamInt {
    /// Create a new integer parameter
    fn new(value: i32) -> Self {
        ParamInt { value }
    }

    /// Encode the parameter
    fn encode(&self) -> u32 {
        ((self.value << 1) ^ (self.value >> 31)) as u32
    }
}

impl<F> GeomEncoder<F>
where
    F: Float,
{
    /// Create a new geometry encoder.
    ///
    /// * `geom_tp` Geometry type.
    pub fn new(geom_tp: GeomType) -> Self {
        GeomEncoder {
            geom_tp,
            x_min: i32::MIN,
            x_max: i32::MAX,
            y_min: i32::MIN,
            y_max: i32::MAX,
            ..Default::default()
        }
    }

    /// Adjust min/max values
    fn adjust_minmax(mut self) -> Self {
        if self.bbox != BBox::default() {
            let p = self.transform * (self.bbox.x_min(), self.bbox.y_min());
            let x = p.x.round().to_i32().unwrap_or(i32::MIN);
            let y = p.y.round().to_i32().unwrap_or(i32::MIN);
            self.x_min = x;
            self.y_min = y;
            let p = self.transform * (self.bbox.x_max(), self.bbox.y_max());
            let x = p.x.round().to_i32().unwrap_or(i32::MAX);
            let y = p.y.round().to_i32().unwrap_or(i32::MAX);
            self.x_max = x;
            self.y_max = y;
        }
        self
    }

    /// Add a bounding box
    pub fn bbox(mut self, bbox: BBox<F>) -> Self {
        self.bbox = bbox;
        self.adjust_minmax()
    }

    /// Add a transform
    pub fn transform(mut self, transform: Transform<F>) -> Self {
        self.transform = transform;
        self.adjust_minmax()
    }

    /// Add a Command
    fn command(&mut self, cmd: Command, count: u32) {
        log::trace!("command: {cmd:?}, count: {count}");
        self.cmd_offset = self.data.len();
        self.data.push(CommandInt::new(cmd, count).encode());
    }

    /// Set count of the most recent Command.
    fn set_command(&mut self, cmd: Command, count: u32) {
        let off = self.cmd_offset;
        self.data[off] = CommandInt::new(cmd, count).encode();
    }

    /// Make point with tile coörindates.
    fn make_point(&self, x: F, y: F) -> Result<(i32, i32)> {
        let p = self.transform * (x, y);
        let mut x = p.x.round().to_i32().ok_or(Error::InvalidValue())?;
        let mut y = p.y.round().to_i32().ok_or(Error::InvalidValue())?;
        // FIXME: clipping to the bounding box is technically incorrect;
        //        we should find the intersection point when crossing it
        if self.x_min <= self.x_max {
            x = x.clamp(self.x_min, self.x_max);
        } else {
            x = x.clamp(self.x_max, self.x_min);
        }
        if self.y_min <= self.y_max {
            y = y.clamp(self.y_min, self.y_max);
        } else {
            y = y.clamp(self.y_max, self.y_min);
        }
        Ok((x, y))
    }

    /// Push one point with relative coörindates.
    fn push_point(&mut self, x: i32, y: i32) {
        log::trace!("push_point: {x},{y}");
        let (px, py) = self.pt.unwrap_or((0, 0));
        self.data.push(ParamInt::new(x.saturating_sub(px)).encode());
        self.data.push(ParamInt::new(y.saturating_sub(py)).encode());
        self.prev_pt = self.pt;
        self.pt = Some((x, y));
    }

    /// Overwrite current point.
    fn should_simplify_point(&self, x: i32, y: i32) -> bool {
        if let (Some((ppx, ppy)), Some((px, py))) = (self.prev_pt, self.pt) {
            if ppx == px && px == x {
                return (ppy < py && py < y) || (ppy > py && py > y);
            }
            if ppy == py && py == y {
                return (ppx < px && px < x) || (ppx > px && px > x);
            }
        }
        false
    }

    /// Overwrite current point.
    fn overwrite_point(&mut self, x: i32, y: i32) {
        log::trace!("overwrite_point: {x},{y}");
        debug_assert!(self.count > 1);
        debug_assert!(self.data.len() > 1);
        // first, remove current point
        self.data.truncate(self.data.len() - 2);
        let (px, py) = self.prev_pt.unwrap();
        self.data.push(ParamInt::new(x.saturating_sub(px)).encode());
        self.data.push(ParamInt::new(y.saturating_sub(py)).encode());
        self.pt = Some((x, y));
    }

    /// Add a point.
    pub fn add_point(&mut self, x: F, y: F) -> Result<()> {
        if self.count == 0 {
            self.prev_pt = None;
        }
        let (x, y) = self.make_point(x, y)?;
        if let Some((px, py)) = self.pt {
            if x == px && y == py {
                log::trace!("redundant point: {x},{y}");
                return Ok(());
            }
        }
        if self.should_simplify_point(x, y) {
            self.overwrite_point(x, y);
            return Ok(());
        }
        match self.geom_tp {
            GeomType::Point => {
                if self.count == 0 {
                    self.command(Command::MoveTo, 1);
                }
            }
            GeomType::Linestring => match self.count {
                0 => self.command(Command::MoveTo, 1),
                1 => self.command(Command::LineTo, 1),
                _ => (),
            },
            GeomType::Polygon => match self.count {
                0 => self.command(Command::MoveTo, 1),
                1 => self.command(Command::LineTo, 1),
                _ => (),
            },
        }
        self.push_point(x, y);
        self.count += 1;
        Ok(())
    }

    /// Add a point, taking ownership (for method chaining).
    pub fn point(mut self, x: F, y: F) -> Result<Self> {
        self.add_point(x, y)?;
        Ok(self)
    }

    /// Complete the current geometry (for multilinestring / multipolygon).
    pub fn complete_geom(&mut self) -> Result<()> {
        // FIXME: return Error::InvalidGeometry
        //        if "MUST" rules in the spec are violated
        match self.geom_tp {
            GeomType::Point => (),
            GeomType::Linestring => {
                if self.count > 1 {
                    self.set_command(Command::LineTo, self.count - 1);
                }
                self.count = 0;
            }
            GeomType::Polygon => {
                if self.count > 1 {
                    self.set_command(Command::LineTo, self.count - 1);
                    self.command(Command::ClosePath, 1);
                }
                self.count = 0;
            }
        }
        Ok(())
    }

    /// Complete the current geometry (for multilinestring / multipolygon).
    pub fn complete(mut self) -> Result<Self> {
        self.complete_geom()?;
        Ok(self)
    }

    /// Encode the geometry data, consuming the encoder.
    pub fn encode(mut self) -> Result<GeomData> {
        // FIXME: return Error::InvalidGeometry
        //        if "MUST" rules in the spec are violated
        self = if let GeomType::Point = self.geom_tp {
            if self.count > 1 {
                self.set_command(Command::MoveTo, self.count);
            }
            self
        } else {
            self.complete()?
        };
        Ok(GeomData::new(self.geom_tp, self.data))
    }
}

impl GeomData {
    /// Create new geometry data.
    ///
    /// * `geom_tp` Geometry type.
    /// * `data` Validated geometry.
    fn new(geom_tp: GeomType, data: Vec<u32>) -> Self {
        GeomData { geom_tp, data }
    }

    /// Get the geometry type
    pub(crate) fn geom_type(&self) -> GeomType {
        self.geom_tp
    }

    /// Check if data is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get length of data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Get the geometry data
    pub(crate) fn into_vec(self) -> Vec<u32> {
        self.data
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Examples from MVT spec:
    #[test]
    fn test_point() {
        let v = GeomEncoder::new(GeomType::Point)
            .point(25.0, 17.0)
            .unwrap()
            .encode()
            .unwrap()
            .into_vec();
        assert_eq!(v, vec!(9, 50, 34));
    }

    #[test]
    fn test_multipoint() {
        let v = GeomEncoder::new(GeomType::Point)
            .point(5.0, 7.0)
            .unwrap()
            .point(3.0, 2.0)
            .unwrap()
            .encode()
            .unwrap()
            .into_vec();
        assert_eq!(v, vec!(17, 10, 14, 3, 9));
    }

    #[test]
    fn test_linestring() {
        let v = GeomEncoder::new(GeomType::Linestring)
            .point(2.0, 2.0)
            .unwrap()
            .point(2.0, 10.0)
            .unwrap()
            .point(10.0, 10.0)
            .unwrap()
            .encode()
            .unwrap()
            .into_vec();
        assert_eq!(v, vec!(9, 4, 4, 18, 0, 16, 16, 0));
    }

    #[test]
    fn test_multilinestring() {
        let v = GeomEncoder::new(GeomType::Linestring)
            .point(2.0, 2.0)
            .unwrap()
            .point(2.0, 10.0)
            .unwrap()
            .point(10.0, 10.0)
            .unwrap()
            .complete()
            .unwrap()
            .point(1.0, 1.0)
            .unwrap()
            .point(3.0, 5.0)
            .unwrap()
            .encode()
            .unwrap()
            .into_vec();
        assert_eq!(v, vec!(9, 4, 4, 18, 0, 16, 16, 0, 9, 17, 17, 10, 4, 8));
    }

    #[test]
    fn test_polygon() {
        let v = GeomEncoder::new(GeomType::Polygon)
            .point(3.0, 6.0)
            .unwrap()
            .point(8.0, 12.0)
            .unwrap()
            .point(20.0, 34.0)
            .unwrap()
            .encode()
            .unwrap()
            .into_vec();
        assert_eq!(v, vec!(9, 6, 12, 18, 10, 12, 24, 44, 15));
    }

    #[test]
    fn test_multipolygon() {
        let v = GeomEncoder::new(GeomType::Polygon)
            // positive area => exterior ring
            .point(0.0, 0.0)
            .unwrap()
            .point(10.0, 0.0)
            .unwrap()
            .point(10.0, 10.0)
            .unwrap()
            .point(0.0, 10.0)
            .unwrap()
            .complete()
            .unwrap()
            // positive area => exterior ring
            .point(11.0, 11.0)
            .unwrap()
            .point(20.0, 11.0)
            .unwrap()
            .point(20.0, 20.0)
            .unwrap()
            .point(11.0, 20.0)
            .unwrap()
            .complete()
            .unwrap()
            // negative area => interior ring
            .point(13.0, 13.0)
            .unwrap()
            .point(13.0, 17.0)
            .unwrap()
            .point(17.0, 17.0)
            .unwrap()
            .point(17.0, 13.0)
            .unwrap()
            .encode()
            .unwrap()
            .into_vec();
        assert_eq!(
            v,
            vec!(
                9, 0, 0, 26, 20, 0, 0, 20, 19, 0, 15, 9, 22, 2, 26, 18, 0, 0,
                18, 17, 0, 15, 9, 4, 13, 26, 0, 8, 8, 0, 0, 7, 15
            )
        );
    }
}
