//! Rainmill Geo: generic geospatial primitives.
//!
//! Geometry storage, point-in-polygon resolution, distance and nearest-neighbour
//! queries, and GeoJSON exchange, over PostGIS. It is reusable by any spatial
//! domain (a store locator, delivery zones) and knows nothing about the place
//! tree; an app composes it with the Rainmill Location plugin by foreign key.
//!
//! Requires the `postgis` database capability, declared below and checked at
//! boot. The geometry helpers, queries, and GeoJSON support land incrementally.

use laterite_core::{Capability, Module, ModuleId};

/// The Rainmill Geo plugin.
pub struct GeoModule;

impl Module for GeoModule {
    fn id(&self) -> ModuleId {
        ModuleId::new("rainmill.geo")
    }

    fn requires_db_capabilities(&self) -> &'static [Capability] {
        const CAPS: &[Capability] = &[Capability::new("postgis")];
        CAPS
    }
}
