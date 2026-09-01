//! The spatial query API over the shape store.
//!
//! Geometry crosses the boundary as text: callers pass GeoJSON in and read
//! GeoJSON out, so the portable `Any` pool never has to decode a binary geometry
//! type. Coordinates are WGS 84 lon/lat (SRID 4326); distances are metres,
//! measured on the spheroid via PostGIS `geography`.
//!
//! This is the coherent first set of operations (the ones an app pulls first:
//! store a shape, read it, point-in-polygon, nearest-neighbour, distance,
//! within-radius). Measurement and transform helpers (area, length, centroid,
//! buffer) and the "helpers over your own geometry column" form follow as they
//! are exercised.

use laterite_core::Db;
use sqlx::Row;

/// A nearest-neighbour hit: the shape's id and its distance from the query point
/// in metres.
#[derive(Debug, Clone, PartialEq)]
pub struct Near {
    pub id: i64,
    pub meters: f64,
}

/// Stores a shape from a GeoJSON geometry, returning its id. `kind` is a free
/// caller label (for example `"boundary"` or `"zone"`).
pub async fn insert_shape(db: &Db, kind: &str, geojson: &str) -> Result<i64, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let row = sqlx::query(
        "INSERT INTO rainmill_geo_shape (kind, geom, created_at, updated_at) \
         VALUES ($1, ST_SetSRID(ST_GeomFromGeoJSON($2), 4326), $3, $3) \
         RETURNING id",
    )
    .bind(kind)
    .bind(geojson)
    .bind(&now)
    .fetch_one(&db.pool)
    .await?;
    row.try_get::<i64, _>("id")
}

/// Reads a shape's geometry back as GeoJSON, or `None` if no such shape.
pub async fn shape_geojson(db: &Db, id: i64) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query("SELECT ST_AsGeoJSON(geom) AS gj FROM rainmill_geo_shape WHERE id = $1")
        .bind(id)
        .fetch_optional(&db.pool)
        .await?;
    row.map(|r| r.try_get::<String, _>("gj")).transpose()
}

/// The ids of shapes that contain the point (point-in-polygon), ascending.
pub async fn containing(db: &Db, lon: f64, lat: f64) -> Result<Vec<i64>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id FROM rainmill_geo_shape \
         WHERE ST_Contains(geom, ST_SetSRID(ST_MakePoint($1, $2), 4326)) \
         ORDER BY id",
    )
    .bind(lon)
    .bind(lat)
    .fetch_all(&db.pool)
    .await?;
    rows.iter().map(|r| r.try_get::<i64, _>("id")).collect()
}

/// The `limit` shapes nearest to the point, closest first, each with its
/// distance in metres. Uses the GiST KNN operator for the ordering.
pub async fn nearest(db: &Db, lon: f64, lat: f64, limit: i64) -> Result<Vec<Near>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, ST_Distance(geom::geography, pt.g::geography) AS m \
         FROM rainmill_geo_shape, \
              (SELECT ST_SetSRID(ST_MakePoint($1, $2), 4326) AS g) AS pt \
         ORDER BY geom <-> pt.g \
         LIMIT $3",
    )
    .bind(lon)
    .bind(lat)
    .bind(limit)
    .fetch_all(&db.pool)
    .await?;
    rows.iter()
        .map(|r| {
            Ok(Near {
                id: r.try_get::<i64, _>("id")?,
                meters: r.try_get::<f64, _>("m")?,
            })
        })
        .collect()
}

/// The distance in metres from a shape to the point, or `None` if no such shape.
pub async fn distance(db: &Db, id: i64, lon: f64, lat: f64) -> Result<Option<f64>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography) AS m \
         FROM rainmill_geo_shape WHERE id = $3",
    )
    .bind(lon)
    .bind(lat)
    .bind(id)
    .fetch_optional(&db.pool)
    .await?;
    row.map(|r| r.try_get::<f64, _>("m")).transpose()
}

/// The ids of shapes within `meters` of the point, nearest first.
pub async fn within_radius(
    db: &Db,
    lon: f64,
    lat: f64,
    meters: f64,
) -> Result<Vec<i64>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id FROM rainmill_geo_shape \
         WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3) \
         ORDER BY geom <-> ST_SetSRID(ST_MakePoint($1, $2), 4326)",
    )
    .bind(lon)
    .bind(lat)
    .bind(meters)
    .fetch_all(&db.pool)
    .await?;
    rows.iter().map(|r| r.try_get::<i64, _>("id")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use laterite_core::{Db, DbBackend};
    use sqlx::any::AnyPoolOptions;

    /// A throwaway PostGIS database for one test: created fresh, extension
    /// enabled, migrations applied, and dropped on `cleanup`.
    struct Ephemeral {
        db: Db,
        maintenance: String,
        name: String,
    }

    impl Ephemeral {
        async fn cleanup(self) {
            let Ephemeral {
                db,
                maintenance,
                name,
            } = self;
            db.pool.close().await;
            let admin = AnyPoolOptions::new()
                .max_connections(1)
                .connect(&maintenance)
                .await
                .unwrap();
            let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS {name} WITH (FORCE)"))
                .execute(&admin)
                .await;
            admin.close().await;
        }
    }

    /// Spins up an ephemeral PostGIS database, or `None` when no Postgres
    /// maintenance URL is set (so the spatial suite skips on the default SQLite
    /// dev run and only runs where PostGIS is available, e.g. the plugin's CI).
    async fn postgis_db() -> Option<Ephemeral> {
        let maintenance = std::env::var("LATERITE_TEST_DATABASE_URL").ok()?;
        sqlx::any::install_default_drivers();
        if DbBackend::from_url(&maintenance).ok()? != DbBackend::Postgres {
            return None;
        }
        let name = format!("rainmill_geo_test_{}", uuid::Uuid::new_v4().simple());
        let admin = AnyPoolOptions::new()
            .max_connections(1)
            .connect(&maintenance)
            .await
            .unwrap();
        sqlx::query(&format!("CREATE DATABASE {name}"))
            .execute(&admin)
            .await
            .unwrap();
        admin.close().await;

        let cut = maintenance.rfind('/').expect("maintenance url has a path");
        let url = format!("{}/{name}", &maintenance[..cut]);
        let pool = AnyPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .unwrap();
        sqlx::query("CREATE EXTENSION IF NOT EXISTS postgis")
            .execute(&pool)
            .await
            .unwrap();
        let db = Db::new(pool, DbBackend::Postgres);
        laterite_core::migration::run(&db.pool, db.backend, &[crate::migrations::migrations()])
            .await
            .unwrap();
        Some(Ephemeral {
            db,
            maintenance,
            name,
        })
    }

    #[tokio::test]
    async fn spatial_ops_over_the_shape_store() {
        let Some(env) = postgis_db().await else {
            eprintln!("skipping: set LATERITE_TEST_DATABASE_URL to a Postgres URL to run");
            return;
        };
        let db = &env.db;
        // A square around Bengaluru, in lon/lat.
        let poly = r#"{"type":"Polygon","coordinates":[[[77.5,12.9],[77.7,12.9],[77.7,13.0],[77.5,13.0],[77.5,12.9]]]}"#;
        let id = insert_shape(db, "zone", poly).await.unwrap();

        // Point-in-polygon: a point inside hits, one far outside misses.
        assert_eq!(containing(db, 77.6, 12.95).await.unwrap(), vec![id]);
        assert!(containing(db, 78.5, 13.5).await.unwrap().is_empty());

        // Geometry round-trips as GeoJSON.
        let gj = shape_geojson(db, id).await.unwrap().unwrap();
        assert!(gj.contains("Polygon"));

        // Nearest returns the shape at ~0 m (the query point is inside it).
        let near = nearest(db, 77.6, 12.95, 5).await.unwrap();
        assert_eq!(near.first().map(|n| n.id), Some(id));
        assert!(near[0].meters < 1.0);

        // Within-radius and distance agree with point-in-polygon.
        assert_eq!(
            within_radius(db, 77.6, 12.95, 100.0).await.unwrap(),
            vec![id]
        );
        assert!(distance(db, id, 77.6, 12.95).await.unwrap().unwrap() < 1.0);

        env.cleanup().await;
    }
}
