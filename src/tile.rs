// tile.rs
//
// Copyright (c) 2019-2026  Minnesota Department of Transportation
//
//! Tile, Layer and Feature structs.
//!
use crate::encoder::{GeomData, GeomType};
use crate::error::{Error, Result};
use crate::vector_tile::Tile as VecTile;
use crate::vector_tile::tile::{
    Feature as VtFeature, GeomType as VtGeomType, Layer as VtLayer, Value,
};
use prost::Message;
use std::collections::HashMap;
use std::io::Write;

/// A tile represents a rectangular region of a map.
///
/// Each tile can contain any number of [layers].  When all layers have been
/// added to the tile, it can be [written out] or [converted] to a `Vec<u8>`.
///
/// # Example
/// ```
/// # use mvt::Error;
/// # fn main() -> Result<(), Error> {
/// use mvt::Tile;
///
/// let mut tile = Tile::new(4096);
/// let layer = tile.create_layer("First Layer");
/// // ...
/// // set up the layer
/// // ...
/// tile.add_layer(layer)?;
/// // ...
/// // add more layers
/// // ...
/// let data = tile.to_bytes()?;
/// # Ok(())
/// # }
/// ```
///
/// [converted]: struct.Tile.html#method.to_bytes
/// [layers]: struct.Layer.html
/// [written out]: struct.Tile.html#method.write_to
pub struct Tile {
    vec_tile: VecTile,
    extent: u32,
}

/// Represents a single MVT attribute value of arbitrary MVT-supported type.
///
/// This is a internal-only data structure, used as a building block to track unique values in a
/// layer in order to speed up MVT tile generation for tiles with many different attribute values.
#[derive(Eq, Hash, PartialEq)]
enum ValueKey {
    String(String),
    Float(u32),
    Double(u64),
    Int(i64),
    Uint(u64),
    Sint(i64),
    Bool(bool),
    Empty,
}

impl From<&Value> for ValueKey {
    fn from(value: &Value) -> Self {
        if let Some(value) = &value.string_value {
            return Self::String(value.clone());
        }
        if let Some(value) = value.float_value {
            return Self::Float(if value == 0.0 { 0 } else { value.to_bits() });
        }
        if let Some(value) = value.double_value {
            return Self::Double(if value == 0.0 {
                0
            } else {
                value.to_bits()
            });
        }
        if let Some(value) = value.int_value {
            return Self::Int(value);
        }
        if let Some(value) = value.uint_value {
            return Self::Uint(value);
        }
        if let Some(value) = value.sint_value {
            return Self::Sint(value);
        }
        if let Some(value) = value.bool_value {
            return Self::Bool(value);
        }
        Self::Empty
    }
}

/// A layer is a set of related features in a tile.
///
/// # Example
/// ```
/// use mvt::Tile;
///
/// let mut tile = Tile::new(4096);
/// let layer = tile.create_layer("First Layer");
/// // ...
/// // set up the layer
/// // ...
/// ```
pub struct Layer {
    layer: VtLayer,
    key_indices: HashMap<String, usize>,
    value_indices: HashMap<ValueKey, usize>,
}

/// A Feature contains map geometry with related metadata.
///
/// A new Feature can be obtained with [Layer.into_feature].
/// After optionally adding an ID and tags, retrieve the Layer with the Feature
/// added by calling [Feature.into_layer].
///
/// # Example
/// ```
/// # use mvt::Error;
/// # fn main() -> Result<(), Error> {
/// use mvt::{GeomEncoder, GeomType, Tile};
/// use pointy::Transform;
///
/// let tile = Tile::new(4096);
/// let layer = tile.create_layer("First Layer");
/// let geom_data = GeomEncoder::new(GeomType::Point)
///     .point(1.0, 2.0)?
///     .point(7.0, 6.0)?
///     .encode()?;
/// let feature = layer.into_feature(geom_data);
/// // ...
/// // add any tags or ID to the feature
/// // ...
/// let layer = feature.into_layer();
/// # Ok(())
/// # }
/// ```
///
/// [Layer.into_feature]: struct.Layer.html#method.into_feature
/// [Feature.into_layer]: struct.Feature.html#method.into_layer
pub struct Feature {
    feature: VtFeature,
    layer: Layer,
    num_keys: usize,
    num_values: usize,
}

impl Tile {
    /// Create a new tile.
    ///
    /// * `extent` Height / width of tile bounds.
    pub fn new(extent: u32) -> Self {
        let vec_tile = VecTile::default();
        Tile { vec_tile, extent }
    }

    /// Get extent, or height / width of tile bounds.
    pub fn extent(&self) -> u32 {
        self.extent
    }

    /// Get the number of layers.
    pub fn num_layers(&self) -> usize {
        self.vec_tile.layers.len()
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
    pub fn add_layer(&mut self, layer: Layer) -> Result<()> {
        if layer.layer.extent != Some(self.extent) {
            return Err(Error::WrongExtent());
        }
        if self
            .vec_tile
            .layers
            .iter()
            .any(|n| n.name == layer.layer.name)
        {
            Err(Error::DuplicateName())
        } else {
            self.vec_tile.layers.push(layer.layer);
            Ok(())
        }
    }

    /// Write the tile.
    ///
    /// * `out` Writer to output the tile.
    pub fn write_to(&self, out: &mut dyn Write) -> Result<()> {
        out.write_all(&self.vec_tile.encode_to_vec())?;
        Ok(())
    }

    /// Encode the tile and return the bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.vec_tile.encode_to_vec())
    }

    /// Compute the encoded size in bytes.
    pub fn compute_size(&self) -> usize {
        self.vec_tile.encoded_len()
    }
}

impl Default for Layer {
    fn default() -> Self {
        let layer = VtLayer::default();
        Layer {
            layer,
            key_indices: HashMap::new(),
            value_indices: HashMap::new(),
        }
    }
}

impl Layer {
    /// Create a new layer.
    ///
    /// * `name` Layer name.
    /// * `extent` Width / height of tile bounds.
    fn new(name: &str, extent: u32) -> Self {
        let layer = VtLayer {
            version: 2,
            name: name.to_string(),
            extent: Some(extent),
            ..Default::default()
        };
        Layer {
            layer,
            key_indices: HashMap::new(),
            value_indices: HashMap::new(),
        }
    }

    /// Get the layer name.
    pub fn name(&self) -> Option<&str> {
        Some(&self.layer.name)
    }

    /// Get number of features (count).
    pub fn num_features(&self) -> usize {
        self.layer.features.len()
    }

    /// Create a new feature, giving it ownership of the layer.
    ///
    /// * `geom_data` Geometry data (consumed by this method).
    pub fn into_feature(self, geom_data: GeomData) -> Feature {
        let num_keys = self.layer.keys.len();
        let num_values = self.layer.values.len();
        let feature = VtFeature {
            r#type: Some(match geom_data.geom_type() {
                GeomType::Point => VtGeomType::Point as i32,
                GeomType::Linestring => VtGeomType::Linestring as i32,
                GeomType::Polygon => VtGeomType::Polygon as i32,
            }),
            geometry: geom_data.into_vec(),
            ..Default::default()
        };
        Feature {
            feature,
            layer: self,
            num_keys,
            num_values,
        }
    }

    /// Get position of a key in the layer keys.  If the key is not found, it
    /// is added as the last key.
    fn key_pos(&mut self, key: &str) -> usize {
        if let Some(&index) = self.key_indices.get(key) {
            return index;
        }
        let index = self.layer.keys.len();
        let key = key.to_string();
        self.layer.keys.push(key.clone());
        self.key_indices.insert(key, index);
        index
    }

    /// Get position of a value in the layer values.  If the value is not found,
    /// it is added as the last value.
    fn val_pos(&mut self, value: Value) -> usize {
        let value_key = ValueKey::from(&value);
        if let Some(&index) = self.value_indices.get(&value_key) {
            return index;
        }
        let index = self.layer.values.len();
        self.layer.values.push(value);
        self.value_indices.insert(value_key, index);
        index
    }
}

impl Feature {
    /// Complete the feature, returning ownership of the layer.
    pub fn into_layer(mut self) -> Layer {
        self.layer.layer.features.push(self.feature);
        self.layer
    }

    /// Get the layer, abandoning the feature.
    pub fn layer(mut self) -> Layer {
        // Reset key/value lengths
        self.layer.layer.keys.truncate(self.num_keys);
        self.layer.layer.values.truncate(self.num_values);
        // Remove any key/value indices added by this feature, so the
        // HashMaps stay in sync with the truncated keys/values Vecs.
        let num_keys = self.num_keys;
        let num_values = self.num_values;
        self.layer.key_indices.retain(|_, idx| *idx < num_keys);
        self.layer.value_indices.retain(|_, idx| *idx < num_values);
        self.layer
    }

    /// Set the feature ID.
    pub fn set_id(&mut self, id: u64) {
        let layer = &self.layer.layer;
        if cfg!(debug_assertions)
            && layer.features.iter().any(|f| f.id == Some(id))
        {
            log::warn!("Duplicate feature ID ({id}) in layer {:?}", layer.name);
        }
        self.feature.id = Some(id);
    }

    /// Get number of tags (count).
    pub fn num_tags(&self) -> usize {
        self.feature.tags.len()
    }

    /// Add a tag of string type.
    pub fn add_tag_string(&mut self, key: &str, val: &str) {
        let value = Value {
            string_value: Some(val.to_string()),
            ..Default::default()
        };
        self.add_tag(key, value);
    }

    /// Add a tag of double type.
    pub fn add_tag_double(&mut self, key: &str, val: f64) {
        let value = Value {
            double_value: Some(val),
            ..Default::default()
        };
        self.add_tag(key, value);
    }

    /// Add a tag of float type.
    pub fn add_tag_float(&mut self, key: &str, val: f32) {
        let value = Value {
            float_value: Some(val),
            ..Default::default()
        };
        self.add_tag(key, value);
    }

    /// Add a tag of int type.
    pub fn add_tag_int(&mut self, key: &str, val: i64) {
        let value = Value {
            int_value: Some(val),
            ..Default::default()
        };
        self.add_tag(key, value);
    }

    /// Add a tag of uint type.
    pub fn add_tag_uint(&mut self, key: &str, val: u64) {
        let value = Value {
            uint_value: Some(val),
            ..Default::default()
        };
        self.add_tag(key, value);
    }

    /// Add a tag of sint type.
    pub fn add_tag_sint(&mut self, key: &str, val: i64) {
        let value = Value {
            sint_value: Some(val),
            ..Default::default()
        };
        self.add_tag(key, value);
    }

    /// Add a tag of bool type.
    pub fn add_tag_bool(&mut self, key: &str, val: bool) {
        let value = Value {
            bool_value: Some(val),
            ..Default::default()
        };
        self.add_tag(key, value);
    }

    /// Add a tag.
    fn add_tag(&mut self, key: &str, value: Value) {
        let kidx = self.layer.key_pos(key);
        self.feature.tags.push(kidx as u32);
        let vidx = self.layer.val_pos(value);
        self.feature.tags.push(vidx as u32);
    }
}
