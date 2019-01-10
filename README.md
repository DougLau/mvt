# mvt
A library for encoding [mapbox vector tiles](https://github.com/mapbox/vector-tile-spec)
(MVT).  Version 2.1 of the standard is supported.

## Example
```rust
use mvt::{GeomEncoder,GeomType,Tile,Transform}

let mut tile = Tile::new(2048);
let layer = tile.create_layer("First Layer");
let encoder = GeomEncoder::new(GeomType::Linestring, Transform::new());
let mut feature = layer.into_feature(encoder);
feature.set_id(1);
feature.add_tag_string("key", "value");
let layer = feature.into_layer();
tile.add_layer(layer)?;
let data = tile.to_bytes()?;
```
