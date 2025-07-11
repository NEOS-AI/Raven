# H3 GeoSpatial Indexing

We are using [Uber H3](https://h3geo.org/) for geospatial indexing.
H3 does not support Rust officially, but there is a translation to Rust ([h3o](https://github.com/HydroniumLabs/h3o))

## Strategy

- Use [`latLngToCell`](https://h3geo.org/docs/api/indexing/#latlngtocell) to convert lat-lng coordinates to H3 cells.
- Use [`gridDisk`](https://h3geo.org/docs/api/traversal#griddisk) to get the Top-K H3 cells around a given cell.
- Use [`cellToLatLng`](https://h3geo.org/docs/api/indexing/#celltolatlng) to convert H3 cells back to lat-lng coordinates.
