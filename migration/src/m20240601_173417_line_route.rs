use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        let db = manager.get_connection();
        db.execute_unprepared("CREATE EXTENSION IF NOT EXISTS postgis").await.unwrap();
        db.execute_unprepared("
            create table route (
                id SERIAL PRIMARY KEY,
                imei VARCHAR(16) NOT NULL,
                points JSON,
                route GEOGRAPHY(LINESTRING, 4326),
                exit_tresh_hold integer
            );
        ").await.unwrap();

        Ok(())

    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Route::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Route {
    Table,
    Id,
}