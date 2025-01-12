use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Blacklist::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Blacklist::Token).string().not_null().primary_key())
                    .col(ColumnDef::new(Blacklist::ExpiresAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Blacklist::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Blacklist {
    Table,
    Token,
    ExpiresAt,
}