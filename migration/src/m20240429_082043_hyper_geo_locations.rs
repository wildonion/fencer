

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        // creating the hypertable for timescaledb queries, it automatically 
        // adds an index on date column for executing high performance queries
        let db = manager.get_connection();
        db.execute_unprepared("CREATE EXTENSION IF NOT EXISTS postgis").await.unwrap();
        db.execute_unprepared("
            create table hyper_geo_locations (
                id SERIAL PRIMARY KEY,
                imei VARCHAR(16) NOT NULL,
                points JSON,
                geom GEOMETRY(Polygon, 4326)
            );
        ").await.unwrap();
        
        Ok(())

    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(HyperGeoLocations::Table).to_owned())
            .await
    }
}

// defining identifiers that will be used in migration
#[derive(DeriveIden)]
enum HyperGeoLocations{
    Table, // this will be mapped to the table name
    // column names
    // ...
}