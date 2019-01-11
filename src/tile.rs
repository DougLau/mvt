// tile.rs
//
// Copyright (c) 2019 Minnesota Department of Transportation
//
//! Tile, Layer and Feature structs.
//!
use protobuf::stream::CodedOutputStream;
use protobuf::Message;
use std::io::Write;
use std::vec::Vec;

use crate::Error;
use crate::encoder::{GeomData, GeomType};
use crate::vector_tile::Tile as VecTile;
use crate::vector_tile::{Tile_Feature, Tile_GeomType, Tile_Layer, Tile_Value};

/// A tile represents a rectangular region of a map.
/// Each tile can contain any number of [layers](struct.Layer.html).
/// When all layers have been added to the tile, it can be
/// [written out](struct.Tile.html#method.write_to) or
/// [converted](struct.Tile.html#method.to_bytes) to a Vec<u8>.
///
/// # Example
/// ```
/// # use mvt::Error;
/// # fn main() -> Result<(), Error> {
///       use mvt::Tile;
///       let mut tile = Tile::new(4096);
///       let layer = tile.create_layer("First Layer");
///       // ...
///       // set up the layer
///       // ...
///       tile.add_layer(layer)?;
///       // ...
///       // add more layers
///       // ...
///       let data = tile.to_bytes()?;
/// #     Ok(())
/// # }
/// ```
pub struct Tile {
    vec_tile: VecTile,
    extent: u32,
}

/// A layer is a set of related features in a tile.
///
/// # Example
/// ```
/// use mvt::Tile;
/// let mut tile = Tile::new(4096);
/// let layer = tile.create_layer("First Layer");
/// // ...
/// // set up the layer
/// // ...
/// ```
pub struct Layer {
    layer: Tile_Layer,
}

/// Features contain map geometry with related metadata.
///
/// A new Feature can be obtained with
/// [Layer.into_feature](struct.Layer.html#method.into_feature).
/// After optionally adding an ID and tags, retrieve the Layer with the Feature
/// added by calling [Feature.into_layer](struct.Feature.html#method.into_layer).
///
/// # Example
/// ```
/// # use mvt::Error;
/// # fn main() -> Result<(), Error> {
///       use mvt::{GeomEncoder,GeomType,Tile,Transform};
///       let mut tile = Tile::new(4096);
///       let layer = tile.create_layer("First Layer");
///       let geom_data = GeomEncoder::new(GeomType::Point, Transform::new())
///                                   .add_point(1.0, 2.0)
///                                   .add_point(7.0, 6.0)
///                                   .encode()?;
///       let feature = layer.into_feature(geom_data);
///       // ...
///       // add any tags or ID to the feature
///       // ...
///       let layer = feature.into_layer();
/// #     Ok(())
/// # }
/// ```
pub struct Feature {
    feature: Tile_Feature,
    layer: Layer,
}

impl Tile {
    /// Create a new tile.
    ///
    /// * `extent` Size in screen coördinates.
    pub fn new(extent: u32) -> Self {
        let vec_tile = VecTile::new();
        Tile { vec_tile, extent }
    }

    /// Get extent in screen coördinates.
    pub fn extent(&self) -> u32 {
        self.extent
    }

    /// Get the number of layers.
    pub fn num_layers(&self) -> usize {
        self.vec_tile.get_layers().len()
    }

    /// Create a new layer.
    ///
    /// * `name` Layer name.
    pub fn create_layer(&self, name: &str) -> Layer {
        Layer::new(name, self.extent)
    }

    /// Add a layer.
    ///
    /// * `layer` The layer.
    ///
    /// Returns an error if:
    /// * a layer with the same name already exists
    /// * the layer extent does not match the tile extent
    pub fn add_layer(&mut self, layer: Layer) -> Result<(), Error> {
        if layer.layer.get_extent() != self.extent {
            return Err(Error::WrongExtent());
        }
        if self
            .vec_tile
            .get_layers()
            .iter()
            .any({ |n| n.get_name() == layer.layer.get_name() })
        {
            Err(Error::DuplicateName())
        } else {
            self.vec_tile.mut_layers().push(layer.layer);
            Ok(())
        }
    }

    /// Write the tile.
    ///
    /// * `out` Writer to output the tile.
    pub fn write_to(&self, mut out: &mut Write) -> Result<(), Error> {
        let mut os = CodedOutputStream::new(&mut out);
        let _ = self.vec_tile.write_to(&mut os);
        if let Err(e) = os.flush() {
            return Err(Error::Protobuf(e));
        }
        Ok(())
    }

    /// Encode the tile and return the bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut v = Vec::with_capacity(self.compute_size());
        self.write_to(&mut v)?;
        Ok(v)
    }

    /// Compute the encoded size in bytes.
    fn compute_size(&self) -> usize {
        self.vec_tile.compute_size() as usize
    }
}

impl Layer {
    /// Create a new layer.
    fn new(name: &str, extent: u32) -> Self {
        let mut layer = Tile_Layer::new();
        layer.set_version(2);
        layer.set_name(name.to_string());
        layer.set_extent(extent);
        Layer { layer }
    }

    /// Get number of features (count).
    pub fn num_features(&self) -> usize {
        self.layer.get_features().len()
    }

    /// Create a new feature, giving it ownership of the layer.
    ///
    /// * `geom_data` Geometry data (consumed by this method).
    pub fn into_feature(self, geom_data: GeomData) -> Feature {
        let mut feature = Tile_Feature::new();
        feature.set_field_type(match geom_data.geom_type() {
            GeomType::Point => Tile_GeomType::POINT,
            GeomType::Linestring => Tile_GeomType::LINESTRING,
            GeomType::Polygon => Tile_GeomType::POLYGON,
        });
        feature.set_geometry(geom_data.into_vec());
        Feature {
            feature,
            layer: self,
        }
    }

    /// Get position of a key in the layer keys.  If the key is not found, it
    /// is added as the last key.
    fn key_pos(&mut self, key: &str) -> usize {
        self.layer
            .get_keys()
            .iter()
            .position(|k| *k == key)
            .unwrap_or_else(|| {
                self.layer.mut_keys().push(key.to_string());
                self.layer.get_keys().len() - 1
            })
    }

    /// Get position of a value in the layer values.  If the value is not found,
    /// it is added as the last value.
    fn val_pos(&mut self, value: Tile_Value) -> usize {
        self.layer
            .get_values()
            .iter()
            .position(|v| *v == value)
            .unwrap_or_else(|| {
                self.layer.mut_values().push(value);
                self.layer.get_values().len() - 1
            })
    }
}

impl Feature {
    /// Complete the feature, returning ownership of the layer.
    pub fn into_layer(mut self) -> Layer {
        self.layer.layer.mut_features().push(self.feature);
        self.layer
    }

    /// Set the feature ID.
    pub fn set_id(&mut self, id: u64) -> Result<(), Error> {
        if self
            .layer
            .layer
            .get_features()
            .iter()
            .any({ |f| f.get_id() == id })
        {
            Err(Error::DuplicateId())
        } else {
            Ok(self.feature.set_id(id))
        }
    }

    /// Add a tag of string type.
    pub fn add_tag_string(&mut self, key: &str, val: &str) {
        let mut value = Tile_Value::new();
        value.set_string_value(val.to_string());
        self.add_tag(key, value);
    }

    /// Add a tag of double type.
    pub fn add_tag_double(&mut self, key: &str, val: f64) {
        let mut value = Tile_Value::new();
        value.set_double_value(val);
        self.add_tag(key, value);
    }

    /// Add a tag of float type.
    pub fn add_tag_float(&mut self, key: &str, val: f32) {
        let mut value = Tile_Value::new();
        value.set_float_value(val);
        self.add_tag(key, value);
    }

    /// Add a tag of int type.
    pub fn add_tag_int(&mut self, key: &str, val: i64) {
        let mut value = Tile_Value::new();
        value.set_int_value(val);
        self.add_tag(key, value);
    }

    /// Add a tag of uint type.
    pub fn add_tag_uint(&mut self, key: &str, val: u64) {
        let mut value = Tile_Value::new();
        value.set_uint_value(val);
        self.add_tag(key, value);
    }

    /// Add a tag of sint type.
    pub fn add_tag_sint(&mut self, key: &str, val: i64) {
        let mut value = Tile_Value::new();
        value.set_sint_value(val);
        self.add_tag(key, value);
    }

    /// Add a tag of bool type.
    pub fn add_tag_bool(&mut self, key: &str, val: bool) {
        let mut value = Tile_Value::new();
        value.set_bool_value(val);
        self.add_tag(key, value);
    }

    /// Add a tag.
    fn add_tag(&mut self, key: &str, value: Tile_Value) {
        let kidx = self.layer.key_pos(key);
        self.feature.mut_tags().push(kidx as u32);
        let vidx = self.layer.val_pos(value);
        self.feature.mut_tags().push(vidx as u32);
    }
}
