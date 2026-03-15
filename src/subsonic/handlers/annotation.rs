use crate::subsonic::{
    common::{deserialize_optional_bool, deserialize_vec, send_response, SubsonicParams},
    models::{SubsonicResponse, SubsonicResponseBody},
};
use poem::{
    handler,
    web::{Data, Query},
    IntoResponse,
};
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarQuery {
    #[serde(default, deserialize_with = "deserialize_vec")]
    pub id: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_vec")]
    pub album_id: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_vec")]
    pub artist_id: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetRatingQuery {
    pub id: String,
    pub rating: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrobbleQuery {
    pub id: String,
    #[serde(default, deserialize_with = "deserialize_optional_bool")]
    pub submission: Option<bool>,
}

async fn insert_stars(
    db: &DatabaseConnection,
    username: &str,
    query: StarQuery,
) -> Result<(), sea_orm::DbErr> {
    use crate::models::user_star;

    let now = chrono::Utc::now();

    for id in query.id {
        let star_model = user_star::ActiveModel {
            username: Set(username.to_string()),
            item_id: Set(id),
            item_type: Set("song".to_string()),
            starred_at: Set(now),
        };
        user_star::Entity::insert(star_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    user_star::Column::Username,
                    user_star::Column::ItemId,
                    user_star::Column::ItemType,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec_without_returning(db)
            .await?;
    }

    for id in query.album_id {
        let star_model = user_star::ActiveModel {
            username: Set(username.to_string()),
            item_id: Set(id),
            item_type: Set("album".to_string()),
            starred_at: Set(now),
        };
        user_star::Entity::insert(star_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    user_star::Column::Username,
                    user_star::Column::ItemId,
                    user_star::Column::ItemType,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec_without_returning(db)
            .await?;
    }

    for id in query.artist_id {
        let star_model = user_star::ActiveModel {
            username: Set(username.to_string()),
            item_id: Set(id),
            item_type: Set("artist".to_string()),
            starred_at: Set(now),
        };
        user_star::Entity::insert(star_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    user_star::Column::Username,
                    user_star::Column::ItemId,
                    user_star::Column::ItemType,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec_without_returning(db)
            .await?;
    }

    Ok(())
}

async fn remove_stars(
    db: &DatabaseConnection,
    username: &str,
    query: StarQuery,
) -> Result<(), sea_orm::DbErr> {
    use crate::models::user_star;

    if !query.id.is_empty() {
        user_star::Entity::delete_many()
            .filter(user_star::Column::Username.eq(username))
            .filter(user_star::Column::ItemId.is_in(query.id))
            .filter(user_star::Column::ItemType.eq("song"))
            .exec(db)
            .await?;
    }
    if !query.album_id.is_empty() {
        user_star::Entity::delete_many()
            .filter(user_star::Column::Username.eq(username))
            .filter(user_star::Column::ItemId.is_in(query.album_id))
            .filter(user_star::Column::ItemType.eq("album"))
            .exec(db)
            .await?;
    }
    if !query.artist_id.is_empty() {
        user_star::Entity::delete_many()
            .filter(user_star::Column::Username.eq(username))
            .filter(user_star::Column::ItemId.is_in(query.artist_id))
            .filter(user_star::Column::ItemType.eq("artist"))
            .exec(db)
            .await?;
    }
    Ok(())
}

#[handler]
pub async fn star(
    db: Data<&DatabaseConnection>,
    user: Data<&std::sync::Arc<crate::models::user::Model>>,
    params: Data<&SubsonicParams>,
    query: Query<StarQuery>,
) -> impl IntoResponse {
    match insert_stars(db.0, &user.username, query.0).await {
        Ok(_) => send_response(
            SubsonicResponse::new_ok(SubsonicResponseBody::None),
            &params.f,
        ),
        Err(e) => {
            log::error!("Database error in star: {}", e);
            send_response(
                SubsonicResponse::new_error(0, "Failed to star items".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn unstar(
    db: Data<&DatabaseConnection>,
    user: Data<&std::sync::Arc<crate::models::user::Model>>,
    params: Data<&SubsonicParams>,
    query: Query<StarQuery>,
) -> impl IntoResponse {
    match remove_stars(db.0, &user.username, query.0).await {
        Ok(_) => send_response(
            SubsonicResponse::new_ok(SubsonicResponseBody::None),
            &params.f,
        ),
        Err(e) => {
            log::error!("Database error in unstar: {}", e);
            send_response(
                SubsonicResponse::new_error(0, "Failed to unstar items".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn set_rating(
    db: Data<&DatabaseConnection>,
    user: Data<&std::sync::Arc<crate::models::user::Model>>,
    params: Data<&SubsonicParams>,
    query: Query<SetRatingQuery>,
) -> impl IntoResponse {
    use crate::models::{album, artist, child, user_rating};
    use sea_orm::QuerySelect;

    if query.rating < 0 || query.rating > 5 {
        return send_response(
            SubsonicResponse::new_error(0, "Invalid rating".into()),
            &params.f,
        );
    }

    let username = &user.username;

    // Determine item_type by checking all entities concurrently
    let (song_check, album_check, artist_check) = tokio::join!(
        child::Entity::find_by_id(&query.id)
            .select_only()
            .column(child::Column::Id)
            .into_tuple::<String>()
            .one(db.0),
        album::Entity::find_by_id(&query.id)
            .select_only()
            .column(album::Column::Id)
            .into_tuple::<String>()
            .one(db.0),
        artist::Entity::find_by_id(&query.id)
            .select_only()
            .column(artist::Column::Id)
            .into_tuple::<String>()
            .one(db.0),
    );

    let item_type = if song_check.ok().flatten().is_some() {
        "song"
    } else if album_check.ok().flatten().is_some() {
        "album"
    } else if artist_check.ok().flatten().is_some() {
        "artist"
    } else {
        return send_response(
            SubsonicResponse::new_error(70, "Item not found".into()),
            &params.f,
        );
    };

    if query.rating == 0 {
        // Remove rating
        match user_rating::Entity::delete_many()
            .filter(user_rating::Column::Username.eq(username))
            .filter(user_rating::Column::ItemId.eq(&query.id))
            .filter(user_rating::Column::ItemType.eq(item_type))
            .exec(db.0)
            .await
        {
            Ok(_) => send_response(
                SubsonicResponse::new_ok(SubsonicResponseBody::None),
                &params.f,
            ),
            Err(e) => {
                log::error!("Database error in set_rating: {}", e);
                send_response(
                    SubsonicResponse::new_error(0, "Failed to set rating".into()),
                    &params.f,
                )
            }
        }
    } else {
        // Upsert rating
        let rating = user_rating::ActiveModel {
            username: Set(username.to_string()),
            item_id: Set(query.id.clone()),
            item_type: Set(item_type.to_string()),
            rating: Set(query.rating),
        };
        match user_rating::Entity::insert(rating)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    user_rating::Column::Username,
                    user_rating::Column::ItemId,
                    user_rating::Column::ItemType,
                ])
                .update_column(user_rating::Column::Rating)
                .to_owned(),
            )
            .exec_without_returning(db.0)
            .await
        {
            Ok(_) => send_response(
                SubsonicResponse::new_ok(SubsonicResponseBody::None),
                &params.f,
            ),
            Err(e) => {
                log::error!("Database error in set_rating: {}", e);
                send_response(
                    SubsonicResponse::new_error(0, "Failed to set rating".into()),
                    &params.f,
                )
            }
        }
    }
}

#[handler]
pub async fn scrobble(
    db: Data<&DatabaseConnection>,
    params: Data<&SubsonicParams>,
    query: Query<ScrobbleQuery>,
) -> impl IntoResponse {
    use crate::models::{child, now_playing};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};

    let ScrobbleQuery {
        id: song_id,
        submission,
    } = query.0;
    let submission = submission.unwrap_or(false);
    let username = params
        .u
        .as_deref()
        .map(ammonia::clean)
        .unwrap_or_else(|| "guest".to_string());
    let player_name = params
        .c
        .as_deref()
        .map(ammonia::clean)
        .unwrap_or_else(|| "unknown".to_string());

    if !submission {
        let now = chrono::Utc::now();
        let np = now_playing::ActiveModel {
            username: Set(username),
            player_name: Set(player_name),
            song_id: Set(song_id),
            updated_at: Set(now),
        };

        let result = now_playing::Entity::insert(np)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    now_playing::Column::Username,
                    now_playing::Column::PlayerName,
                ])
                .update_columns([now_playing::Column::SongId, now_playing::Column::UpdatedAt])
                .to_owned(),
            )
            .exec(db.0)
            .await;

        if let Err(e) = result {
            log::error!("Database error in update now playing: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Failed to update now playing status".into()),
                &params.f,
            );
        }

        return send_response(
            SubsonicResponse::new_ok(SubsonicResponseBody::None),
            &params.f,
        );
    }

    // submission is true - song finished, remove from now playing
    let _ = now_playing::Entity::delete_many()
        .filter(now_playing::Column::Username.eq(&username))
        .filter(now_playing::Column::PlayerName.eq(&player_name))
        .exec(db.0)
        .await;

    let now = chrono::Utc::now();
    let res = child::Entity::update_many()
        .filter(child::Column::Id.eq(&song_id))
        .col_expr(
            child::Column::PlayCount,
            Expr::col(child::Column::PlayCount).add(1),
        )
        .col_expr(child::Column::LastPlayed, Expr::value(Some(now)))
        .exec(db.0)
        .await;

    match res {
        Ok(_) => send_response(
            SubsonicResponse::new_ok(SubsonicResponseBody::None),
            &params.f,
        ),
        Err(e) => {
            log::error!("Database error in scrobble: {}", e);
            send_response(
                SubsonicResponse::new_error(0, "Failed to scrobble".into()),
                &params.f,
            )
        }
    }
}

#[cfg(test)]
#[path = "annotation_tests.rs"]
mod tests;
