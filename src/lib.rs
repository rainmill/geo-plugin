//! Rainmill Geo: generic geospatial primitives.
//!
//! Geometry storage, point-in-polygon resolution, distance and nearest-neighbour
//! queries, and GeoJSON exchange, over PostGIS. It is reusable by any spatial
//! domain (a store locator, delivery zones) and knows nothing about the place
//! tree; an app composes it with the Rainmill Location plugin by foreign key.
//!
//! Requires the `postgis` database capability, declared below and checked at
//! boot. Measurement/transform helpers and the "over your own column" form land
//! incrementally on top of this first query set (see [`store`]).

mod migrations;
pub mod store;

use laterite_core::{Capability, MigrationSet, Module, ModuleId};

/// This plugin's entry point. Every Laterite plugin exposes `module()`, so the
/// generated `plugins-manifest` collects it without naming the type.
pub fn module() -> Box<dyn Module> {
    Box::new(GeoModule)
}

/// The Rainmill Geo plugin.
pub struct GeoModule;

impl Module for GeoModule {
    fn id(&self) -> ModuleId {
        ModuleId::new(migrations::MODULE_ID)
    }

    fn migrations(&self) -> MigrationSet {
        migrations::migrations()
    }

    fn requires_db_capabilities(&self) -> &'static [Capability] {
        const CAPS: &[Capability] = &[Capability::new("postgis")];
        CAPS
    }
}
