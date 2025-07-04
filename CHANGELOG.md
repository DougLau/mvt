## [Unreleased]

## [0.10.3] - 2025-06-25
### Changed
* Simplify the initial point redundancy handling logic

## [0.10.2] - 2025-06-25
### Fixed
* Fix skipped first line segment in multilinestrings when initial point is
  redundant

## [0.10.0] - 2025-03-15
### Changed
* Rust 2024 edition
* pointy 0.7

## [0.9.4] - 2024-10-23
### Changed
* Updated protobuf to v3.7

## [0.9.3] - 2024-06-06
### Changed
* Updated protobuf to v3.4

## [0.9.0] - 2024-01-11
### Added
* Wgs84Pos / WebMercatorPos conversion structs
* `GeomEncoder::bbox` / `::transform` methods
### Changed
* Updated protobuf to v3.3
### Removed
* Generic parameter for MapGrid -- always use `f64`
* Transform parameter from `GeomEncoder::new`

## [0.8.0] - 2023-01-28
### Added
* `GeomData::is_empty` / `GeomData::len` methods
### Changed
* Moved `BBox` to `pointy` crate
* `GeomEncoder::point` / `add_point` are now fallible (float to int errors)
* `GeomEncoder` now has a `Float` type parameter (`f32` or `f64`)
* `MapGrid` now has a `Float` type parameter (`f32` or `f64`)
* Updated `protobuf` dependency to version 3.2

## [0.7.0] - 2020-09-29
### Changed
* Replaced `geom` module with `pointy` crate dependency

## [0.6.0] - 2020-09-18
### Changed
* Implement Default for Layer
* Made Error enum non-exhaustive
* Replaced `MapGrid::new_web_mercator()` with `MapGrid::default()`

## [0.5.4] - 2020-09-11
### Added
* Use `cargo run --features=update` to update to a new protobuf version
### Changed
* Updated to protobuf 2.17

## [0.5.3] - 2019-10-30
### Changed
* Updated protobuf dependency

## [0.5.2] - 2019-02-28
### Added
* Layer::name() method

## [0.5.1] - 2019-02-22
### Changed
* Made MapGrid cloneable
* Made Tile::compute_size() public

## [0.5.0] - 2019-02-14
### Added
* Feature::layer and ::num_tags methods
* Error::Other
### Changed
* Feature::set_id can no longer fail
### Removed
* Error::DuplicateId

## [0.4.0] - 2019-02-07
### Added
* GeomEncoder::point and ::complete (for method chaining)
### Changed
* GeomEncoder::add_point and ::complete_geom now take a reference

## [0.3.0] - 2019-01-18
### Added
* MapGrid, TileId and BBox
* New error variant: InvalidTid

## [0.2.0] - 2019-01-11
### Added
* Check extent when adding layer to tile
* GeomEncoder now has encode method to create GeomData struct
* New error variant: InvalidGeometry

### Changed
* GeomEncoder now uses builder pattern
* Made Tile::compute_size private
* Tile::get_extent() => extent()

## [0.1.0] - 2019-01-10
* Initial version
