use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CachedEmbeddings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CachedEmbeddings::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CachedEmbeddings::FileHash).string().not_null())
                    .col(ColumnDef::new(CachedEmbeddings::Chunk).text().not_null())
                    .col(ColumnDef::new(CachedEmbeddings::Embedding).array(ColumnType::Float).not_null())
                    .col(ColumnDef::new(CachedEmbeddings::CreatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CachedEmbeddings::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum CachedEmbeddings {
    Table,
    Id,
    FileHash,
    Chunk,
    Embedding,
    CreatedAt,
}