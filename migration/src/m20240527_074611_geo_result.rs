use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        
        let db = manager.get_connection();
        db.execute_unprepared("
            create table geo_result(
                id SERIAL PRIMARY KEY,
                geo_id int,
                imei VARCHAR(16),
                geom_text text,
                points_text text,
                is_contains boolean
            );
        ").await.unwrap();

        db.execute_unprepared("
            ALTER TABLE geo_result ADD FOREIGN KEY(geo_id) REFERENCES hyper_geo_locations(id);
        ").await.unwrap();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GeoResult::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GeoResult {
    Table,
}
