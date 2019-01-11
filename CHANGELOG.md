## [Unreleased]
### Added
* Check extent when adding layer to tile
* GeomEncoder now has encode method to create GeomData struct
* New error variant: InvalidGeometry

### Changed
* GeomEncoder now uses builder pattern
* Made Tile::compute_size private
* Tile::get_extent() => extent()

## [0.1.0] - 2010-01-10
* Initial version
