pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20250315_033605_create_blacklist_table;
mod m20250403_163323_create_cached_embeddings;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250315_033605_create_blacklist_table::Migration),
            Box::new(m20250403_163323_create_cached_embeddings::Migration),
        ]
    }
}
