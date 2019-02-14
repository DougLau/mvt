## [Unreleased]
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
