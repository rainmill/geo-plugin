# Rainmill Geo

A first-party [Laterite](https://github.com/lateritecmf/laterite) plugin: a
**geospatial toolkit** over PostGIS. It gives any application the spatial
operations behind store locators, delivery and service zones, proximity search,
and catchment areas, without writing raw PostGIS SQL.

It works two ways: as **helpers over your own geometry column** (you own the
table, the plugin provides the operations), and as an optional **generic shape
store** for apps that would rather not manage geometry columns themselves. It
knows nothing about any particular domain and composes with
[Rainmill Location](https://github.com/rainmill/location-plugin) by foreign
key.

## Requirements

- Laterite (the module API).
- Database: **PostgreSQL with PostGIS**. The plugin declares the `postgis`
  capability, so an application that registers it refuses to boot on a database
  that cannot provide PostGIS, with a clear error rather than a failure deep
  inside a migration.

## Installation

Add the dependency:

```toml
[dependencies]
rainmill-geo = "0.1"
```

Register the module when building the application:

```rust
laterite_admin::Bootstrap::new("config")
    .module(rainmill_geo::GeoModule)
    .serve()
    .await
```

## What it provides

| Task | Operation |
|---|---|
| "Near me" / store locator | nearest-neighbour, distance, within-radius |
| "Is it in my zone?" / delivery areas, geofencing | point-in-polygon |
| "Draw a radius / catchment" | buffer, area |
| "How big / how long?" | area, length |
| "Center of a region" | centroid |
| "Show on a map / import" | GeoJSON and WKT in/out |

Geometry types supported: point, linestring, polygon, and multipolygon.

Deferred to later releases (documented extension points): intersection and union,
simplification, route boxing, polyline encode/decode, bearing, and geohash.

## Usage

Store a geometry on your own table and query it through the plugin's helpers, or
use the generic shape store:

```rust
// nearest five places to a point
let nearby = geo.nearest(point, 5).await?;
// is a point inside a zone?
let inside = geo.contains(zone_id, point).await?;
// a delivery radius as a polygon
let area = geo.buffer(point, meters(500)).await?;
```

## Status

In active development. The module registration and capability declaration are in
place; the geometry storage, query helpers, and GeoJSON support are landing
incrementally.

## License

Licensed under MIT OR Apache-2.0.
