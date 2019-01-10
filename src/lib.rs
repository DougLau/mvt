// lib.rs      mvt crate.
//
// Copyright (c) 2019 Minnesota Department of Transportation
//
//! A library for encoding [mapbox vector tiles](https://github.com/mapbox/vector-tile-spec)
//! (MVT).  Version 2.1 of the standard is supported.
#[macro_use]
extern crate log;

mod encoder;
mod geom;
mod tile;
mod vector_tile;

pub use crate::encoder::{GeomEncoder,GeomType};
pub use crate::geom::Transform;
pub use crate::tile::{Error,Feature,Layer,Tile};
