pub use sea_orm_migration::prelude::*;

pub use sea_orm_migration::prelude::async_trait::async_trait;

mod m20220101_000001_create_tables;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_tables::Migration),
        ]
    }
}
