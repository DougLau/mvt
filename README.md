# mvt
A library for encoding [mapbox vector tiles](https://github.com/mapbox/vector-tile-spec)
(MVT).  Version 2.1 of the standard is supported.

The API is designed to prevent creating files which are not allowed by the
specification.

## Example

```rust
use mvt::{Error,GeomEncoder,GeomType,Tile,Transform};

fn main() -> Result<(), Error> {
    let mut tile = Tile::new(4096);
    let layer = tile.create_layer("First Layer");
    let b = GeomEncoder::new(GeomType::Linestring, Transform::new())
                        .add_point(0.0, 0.0)
                        .add_point(1024.0, 0.0)
                        .add_point(1024.0, 2048.0)
                        .add_point(2048.0, 2048.0)
                        .add_point(2048.0, 4096.0)
                        .encode()?;
    let mut feature = layer.into_feature(b);
    feature.set_id(1)?;
    feature.add_tag_string("key", "value");
    let layer = feature.into_layer();
    tile.add_layer(layer)?;
    let data = tile.to_bytes()?;
    println!("encoded {} bytes: {:?}", data.len(), data);
    Ok(())
}
```

## Alternatives

These are other rust projects with MVT support:
* [hecate](https://crates.io/crates/hecate)
* [t-rex](https://t-rex.tileserver.ch/)
* [vectortile](https://crates.io/crates/vectortile)
