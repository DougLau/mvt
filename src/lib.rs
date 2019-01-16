// lib.rs      mvt crate.
//
// Copyright (c) 2019 Minnesota Department of Transportation
//
//! A library for encoding [mapbox vector tiles](https://github.com/mapbox/vector-tile-spec)
//! (MVT).
#[macro_use]
extern crate log;

use protobuf::error::ProtobufError;
use std::fmt;

/// MVT Error types
#[derive(Debug)]
pub enum Error {
    /// The tile already contains a layer with the specified name.
    DuplicateName(),
    /// The layer already contains a feature with the specified ID.
    DuplicateId(),
    /// The layer extent does not match the tile extent.
    WrongExtent(),
    /// The tile ID is invalid.
    InvalidTid(),
    /// The geometry does not meet criteria of the specification.
    InvalidGeometry(),
    /// Error while encoding protobuf data.
    Protobuf(ProtobufError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DuplicateName() => write!(f, "Name already exists"),
            Error::DuplicateId() => write!(f, "ID already exists"),
            Error::WrongExtent() => write!(f, "Wrong layer extent"),
            Error::InvalidTid() => write!(f, "Invalid tile ID"),
            Error::InvalidGeometry() => write!(f, "Invalid geometry data"),
            Error::Protobuf(e) => write!(f, "Protobuf {:?}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Protobuf(p) => Some(p),
            _ => None,
        }
    }
}

mod encoder;
mod geom;
mod grid;
mod tile;
mod vector_tile;

pub use crate::encoder::{GeomData, GeomEncoder, GeomType};
pub use crate::geom::Transform;
pub use crate::grid::{BBox, Grid, TileId};
pub use crate::tile::{Feature, Layer, Tile};
