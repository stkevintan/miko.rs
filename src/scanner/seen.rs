use sea_orm::entity::prelude::*;
use sea_orm::Set;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "_scanner_seen")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub struct SeenTracker;

impl SeenTracker {
    pub async fn prepare(db: &DatabaseConnection) -> Result<(), anyhow::Error> {
        db.execute_unprepared("CREATE TABLE IF NOT EXISTS _scanner_seen (id TEXT PRIMARY KEY)")
            .await?;
        db.execute_unprepared("DELETE FROM _scanner_seen").await?;
        Ok(())
    }

    pub async fn insert_batch(
        db: &DatabaseConnection,
        ids: Vec<String>,
    ) -> Result<(), anyhow::Error> {
        if ids.is_empty() {
            return Ok(());
        }

        let models: Vec<ActiveModel> = ids
            .into_iter()
            .map(|id| ActiveModel { id: Set(id) })
            .collect();

        Entity::insert_many(models)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec_without_returning(db)
            .await?;
        Ok(())
    }

    pub async fn clear(db: &DatabaseConnection) -> Result<(), anyhow::Error> {
        db.execute_unprepared("DELETE FROM _scanner_seen").await?;
        Ok(())
    }
}
