use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
       
       let db = manager.get_connection();
        db.execute_unprepared("
            create table route_result(
                id SERIAL PRIMARY KEY,
                route_id int,
                imei VARCHAR(16),
                route_text text,
                points_text text,
                is_contains boolean
            );
        ").await.unwrap();

        db.execute_unprepared("
            ALTER TABLE route_result ADD FOREIGN KEY(route_id) REFERENCES route(id);
        ").await.unwrap();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(RouteResult::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RouteResult {
    Table,
    Id,
}
