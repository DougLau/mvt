// lib.rs      mvt crate.
//
// Copyright (c) 2019 Minnesota Department of Transportation
//
//! A library for encoding [mapbox vector tiles](https://github.com/mapbox/vector-tile-spec)
//! (MVT).
#[macro_use] extern crate log;

mod encoder;
mod error;
mod geom;
mod mapgrid;
mod tile;
mod vector_tile;

pub use crate::encoder::{GeomData, GeomEncoder, GeomType};
pub use crate::error::Error;
pub use crate::geom::{Transform, Vec2};
pub use crate::mapgrid::{BBox, MapGrid, TileId};
pub use crate::tile::{Feature, Layer, Tile};
