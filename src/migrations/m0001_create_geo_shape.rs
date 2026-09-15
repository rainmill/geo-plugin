//! Create the generic shape store.

use laterite_core::strata::*;
// The two irreducibly-PostGIS bits need sea-query's own escapes: the geometry
// column type and the GiST index type.
use sea_query::{Alias, IndexType, IntoIden};

use crate::schema::RainmillGeoShape;

pub struct Migration;

#[async_trait(?Send)]
impl laterite_core::Migration for Migration {
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
