// error.rs
//
// Copyright (c) 2019-2020  Minnesota Department of Transportation
//
use protobuf::error::ProtobufError;
use std::fmt;

/// MVT Error types
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// The tile already contains a layer with the specified name.
    DuplicateName(),
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

impl From<ProtobufError> for Error {
    fn from(e: ProtobufError) -> Self {
        Error::Protobuf(e)
    }
}
