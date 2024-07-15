pub use sea_orm_migration::prelude::*;

mod m20240429_082043_hyper_geo_locations;
mod m20240527_074611_geo_result;
mod m20240605_150651_add_inserted_at;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240429_082043_hyper_geo_locations::Migration),
            Box::new(m20240527_074611_geo_result::Migration),
            Box::new(m20240605_150651_add_inserted_at::Migration),
        ]
    }
}