// tile.rs
//
// Copyright (c) 2019 Minnesota Department of Transportation
//
use protobuf::Message;
use protobuf::error::ProtobufError;
use protobuf::stream::CodedOutputStream;
use std::fmt;
use std::io::Write;
use std::vec::Vec;

use crate::encoder::{GeomEncoder,GeomType};
use crate::vector_tile::Tile as VecTile;
use crate::vector_tile::{Tile_Feature,Tile_GeomType,Tile_Layer,Tile_Value};

#[derive(Debug)]
pub enum Error {
    DuplicateName(),
    DuplicateId(),
    Protobuf(ProtobufError),
}

pub struct Tile {
    vec_tile: VecTile,
    extent: u32,
}

pub struct Layer {
    layer: Tile_Layer,
}

pub struct Feature {
    feature: Tile_Feature,
    layer: Layer,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DuplicateName() => write!(f, "Name already exists"),
            Error::DuplicateId() => write!(f, "ID already exists"),
            Error::Protobuf(_) => write!(f, "Error encoding MVT data"),
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

impl Tile {
    pub fn new(extent: u32) -> Self {
        let vec_tile = VecTile::new();
        Tile { vec_tile, extent }
    }

    pub fn get_extent(&self) -> u32 {
        self.extent
    }

    pub fn num_layers(&self) -> usize {
        self.vec_tile.get_layers().len()
    }

    pub fn create_layer(&self, name: &str) -> Layer {
        Layer::new(name, self.extent)
    }

    pub fn add_layer(&mut self, layer: Layer) -> Result<(), Error> {
        if self.vec_tile.get_layers()
                        .iter()
                        .any({|n| n.get_name() == layer.layer.get_name()})
        {
            Err(Error::DuplicateName())
        } else {
            self.vec_tile.mut_layers().push(layer.layer);
            Ok(())
        }
    }

    pub fn write_to(&self, mut out: &mut Write) -> Result<(), Error> {
        let mut os = CodedOutputStream::new(&mut out);
        let _ = self.vec_tile.write_to(&mut os);
        if let Err(e) = os.flush() {
            return Err(Error::Protobuf(e));
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut v = Vec::with_capacity(self.compute_size() as usize);
        self.write_to(&mut v)?;
        Ok(v)
    }

    pub fn compute_size(&self) -> u32 {
        self.vec_tile.compute_size()
    }
}

impl Layer {
    fn new(name: &str, extent: u32) -> Self {
        let mut layer = Tile_Layer::new();
        layer.set_version(2);
        layer.set_name(name.to_string());
        layer.set_extent(extent);
        Layer { layer }
    }

    pub fn num_features(&self) -> usize {
        self.layer.get_features().len()
    }

    pub fn into_feature(self, encoder: GeomEncoder) -> Feature {
        let mut feature = Tile_Feature::new();
        let geom_tp = match encoder.geom_type() {
            GeomType::Point => Tile_GeomType::POINT,
            GeomType::Linestring => Tile_GeomType::LINESTRING,
            GeomType::Polygon => Tile_GeomType::POLYGON,
        };
        feature.set_field_type(geom_tp);
        feature.set_geometry(encoder.to_vec());
        Feature { feature, layer: self }
    }

    fn key_pos(&mut self, key: &str) -> usize {
        self.layer.get_keys()
                  .iter()
                  .position(|k| *k == key)
                  .unwrap_or_else(||
        {
            self.layer.mut_keys().push(key.to_string());
            self.layer.get_keys().len() - 1
        })
    }

    fn val_pos(&mut self, value: Tile_Value) -> usize {
        self.layer.get_values()
                  .iter()
                  .position(|v| *v == value)
                  .unwrap_or_else(||
        {
            self.layer.mut_values().push(value);
            self.layer.get_values().len() - 1
        })
    }
}

impl Feature {

    pub fn into_layer(mut self) -> Layer {
        self.layer.layer.mut_features().push(self.feature);
        self.layer
    }

    pub fn set_id(&mut self, id: u64) -> Result<(), Error> {
        if self.layer.layer.get_features()
                           .iter()
                           .any({|f| f.get_id() == id})
        {
            Err(Error::DuplicateId())
        } else {
            Ok(self.feature.set_id(id))
        }
    }

    pub fn add_tag_string(&mut self, key: &str, val: &str) {
        let mut value = Tile_Value::new();
        value.set_string_value(val.to_string());
        self.add_tag(key, value);
    }

    pub fn add_tag_double(&mut self, key: &str, val: f64) {
        let mut value = Tile_Value::new();
        value.set_double_value(val);
        self.add_tag(key, value);
    }

    pub fn add_tag_float(&mut self, key: &str, val: f32) {
        let mut value = Tile_Value::new();
        value.set_float_value(val);
        self.add_tag(key, value);
    }

    pub fn add_tag_int(&mut self, key: &str, val: i64) {
        let mut value = Tile_Value::new();
        value.set_int_value(val);
        self.add_tag(key, value);
    }

    pub fn add_tag_uint(&mut self, key: &str, val: u64) {
        let mut value = Tile_Value::new();
        value.set_uint_value(val);
        self.add_tag(key, value);
    }

    pub fn add_tag_sint(&mut self, key: &str, val: i64) {
        let mut value = Tile_Value::new();
        value.set_sint_value(val);
        self.add_tag(key, value);
    }

    pub fn add_tag_bool(&mut self, key: &str, val: bool) {
        let mut value = Tile_Value::new();
        value.set_bool_value(val);
        self.add_tag(key, value);
    }

    fn add_tag(&mut self, key: &str, value: Tile_Value) {
        let kidx = self.layer.key_pos(key);
        self.feature.mut_tags().push(kidx as u32);
        let vidx = self.layer.val_pos(value);
        self.feature.mut_tags().push(vidx as u32);
    }
}
