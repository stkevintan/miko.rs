use crate::subsonic::{
    common::{send_response, SubsonicParams, deserialize_vec},
    models::{SubsonicResponse, SubsonicResponseBody},
};
use poem::{
    handler,
    web::{Data, Query},
    IntoResponse,
};
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
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
    pub submission: Option<bool>,
}

async fn update_starred_status(
    db: &DatabaseConnection,
    query: StarQuery,
    value: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<(), sea_orm::DbErr> {
    use crate::models::{album, artist, child};

    if !query.id.is_empty() {
        child::Entity::update_many()
            .filter(child::Column::Id.is_in(query.id))
            .col_expr(child::Column::Starred, Expr::value(value))
            .exec(db)
            .await?;
    }
    if !query.album_id.is_empty() {
        album::Entity::update_many()
            .filter(album::Column::Id.is_in(query.album_id))
            .col_expr(album::Column::Starred, Expr::value(value))
            .exec(db)
            .await?;
    }
    if !query.artist_id.is_empty() {
        artist::Entity::update_many()
            .filter(artist::Column::Id.is_in(query.artist_id))
            .col_expr(artist::Column::Starred, Expr::value(value))
            .exec(db)
            .await?;
    }
    Ok(())
}

#[handler]
pub async fn star(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
    query: Query<StarQuery>,
) -> impl IntoResponse {
    let now = chrono::Utc::now();
    match update_starred_status(db.0, query.0, Some(now)).await {
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
    params: Query<SubsonicParams>,
    query: Query<StarQuery>,
) -> impl IntoResponse {
    match update_starred_status(db.0, query.0, None).await {
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
    params: Query<SubsonicParams>,
    query: Query<SetRatingQuery>,
) -> impl IntoResponse {
    use crate::models::{album, artist, child};

    if query.rating < 0 || query.rating > 5 {
        return send_response(
            SubsonicResponse::new_error(0, "Invalid rating".into()),
            &params.f,
        );
    }

    match child::Entity::update_many()
        .filter(child::Column::Id.eq(&query.id))
        .col_expr(child::Column::UserRating, Expr::value(query.rating))
        .exec(db.0)
        .await
    {
        Ok(res) => {
            if res.rows_affected > 0 {
                return send_response(
                    SubsonicResponse::new_ok(SubsonicResponseBody::None),
                    &params.f,
                );
            }
        }
        Err(e) => {
            log::error!("Database error in set_rating: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Failed to set rating".into()),
                &params.f,
            );
        }
    }

    // Try album if child not found
    match album::Entity::update_many()
        .filter(album::Column::Id.eq(&query.id))
        .col_expr(album::Column::UserRating, Expr::value(query.rating))
        .exec(db.0)
        .await
    {
        Ok(res) => {
            if res.rows_affected > 0 {
                return send_response(
                    SubsonicResponse::new_ok(SubsonicResponseBody::None),
                    &params.f,
                );
            }
        }
        Err(e) => {
            log::error!("Database error in set_rating: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Failed to set rating".into()),
                &params.f,
            );
        }
    }

    // Try artist if album not found
    match artist::Entity::update_many()
        .filter(artist::Column::Id.eq(&query.id))
        .col_expr(artist::Column::UserRating, Expr::value(query.rating))
        .exec(db.0)
        .await
    {
        Ok(res) => {
            if res.rows_affected > 0 {
                send_response(
                    SubsonicResponse::new_ok(SubsonicResponseBody::None),
                    &params.f,
                )
            } else {
                send_response(
                    SubsonicResponse::new_error(70, "Item not found".into()),
                    &params.f,
                )
            }
        }
        Err(e) => {
            log::error!("Database error in set_rating: {}", e);
            send_response(
                SubsonicResponse::new_error(0, "Failed to set rating".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn scrobble(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
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
