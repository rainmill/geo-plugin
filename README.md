# Rainmill Geo

A first-party [Laterite](https://github.com/lateritecmf/laterite) plugin: generic
geospatial primitives over PostGIS.

Geometry storage, point-in-polygon resolution, distance and nearest-neighbour
queries, and GeoJSON exchange. It is reusable by any spatial domain (a store
locator, delivery zones) and knows nothing about the place tree; an app composes
it with [Rainmill Location](https://github.com/lateritecmf/rainmill-location) by
foreign key. Requires the `postgis` database capability, checked at boot.

## Development

Laterite is unpublished and consumed by path, so check it out beside this repo:

```
parent/
  laterite/
  rainmill-geo/
```

Then `cargo build` and `cargo test` (the test suite needs Postgres with PostGIS).
