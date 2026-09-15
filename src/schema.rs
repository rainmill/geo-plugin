//! Table and column identifiers for this plugin's schema.

use laterite_core::strata::*;

/// The generic shape store: one row per geometry, in SRID 4326 (WGS 84 lon/lat).
/// Apps that own their own geometry column use the query helpers directly instead
/// of this table.
#[derive(Iden)]
pub enum RainmillGeoShape {
    Table,
    Id,
    Kind,
    Geom,
    CreatedAt,
    UpdatedAt,
}
