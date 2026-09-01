//! This plugin's migrations, in apply order.
//!
//! Built with the sea-query schema builder, the same idiom as every other
//! module's migration. Only the two irreducibly-PostGIS bits use sea-query's own
//! escapes: the geometry column via a custom column type, and the GiST index via
//! a custom index type. The `postgis` extension itself is provided by the
//! deployment (the declared capability is verified at boot), not created here.

use laterite_core::strata::*;
use sea_query::{Alias, IndexType, IntoIden};

/// This plugin's module id.
pub const MODULE_ID: &str = "rainmill.geo";

/// The migration set for the geo tables.
pub fn migrations() -> MigrationSet {
    MigrationSet::new(MODULE_ID, vec![Box::new(CreateGeoShape)])
}

/// The generic shape store: one row per geometry, in SRID 4326 (WGS 84 lon/lat).
/// Apps that own their own geometry column use the query helpers directly instead
/// of this table.
#[derive(Iden)]
enum RainmillGeoShape {
    Table,
    Id,
    Kind,
    Geom,
    CreatedAt,
    UpdatedAt,
}

struct CreateGeoShape;

#[async_trait(?Send)]
impl Migration for CreateGeoShape {
    fn name(&self) -> &str {
        "0001_create_geo_shape"
    }

    async fn up(&self, s: &mut Schema<'_>) -> CoreResult<()> {
        s.exec(
            Table::create()
                .table(RainmillGeoShape::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(RainmillGeoShape::Id)
                        .big_integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(RainmillGeoShape::Kind).text().not_null())
                // PostGIS geometry has no portable sea-query type, so name it
                // through the custom-type escape: a WGS 84 (SRID 4326) geometry.
                .col(
                    ColumnDef::new(RainmillGeoShape::Geom)
                        .custom(Alias::new("geometry(Geometry, 4326)"))
                        .not_null(),
                )
                .col(
                    ColumnDef::new(RainmillGeoShape::CreatedAt)
                        .text()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(RainmillGeoShape::UpdatedAt)
                        .text()
                        .not_null(),
                )
                .to_owned(),
        )
        .await?;
        // A GiST index for spatial lookups. GIST is not one of sea-query's
        // portable index kinds, so name it through the custom index-type escape.
        s.exec(
            Index::create()
                .name("rainmill_geo_shape_geom_gist")
                .table(RainmillGeoShape::Table)
                .col(RainmillGeoShape::Geom)
                .index_type(IndexType::Custom(Alias::new("GIST").into_iden()))
                .to_owned(),
        )
        .await
    }

    async fn down(&self, s: &mut Schema<'_>) -> CoreResult<()> {
        s.exec(Table::drop().table(RainmillGeoShape::Table).to_owned())
            .await
    }
}
