use crate::subsonic::{
    common::{send_response, SubsonicParams},
    models::{SubsonicResponse, SubsonicResponseBody},
};
use poem::{
    handler,
    web::{Data, Query},
    IntoResponse,
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use sea_orm::sea_query::Expr;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarQuery {
    #[serde(default)]
    pub id: Vec<String>,
    #[serde(default)]
    pub album_id: Vec<String>,
    #[serde(default)]
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
    use crate::models::{child, album, artist};

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
        Ok(_) => send_response(SubsonicResponse::new_ok(SubsonicResponseBody::None), &params.f),
        Err(e) => {
            log::error!("Database error in star: {}", e);
            send_response(SubsonicResponse::new_error(0, "Failed to star items".into()), &params.f)
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
        Ok(_) => send_response(SubsonicResponse::new_ok(SubsonicResponseBody::None), &params.f),
        Err(e) => {
            log::error!("Database error in unstar: {}", e);
            send_response(SubsonicResponse::new_error(0, "Failed to unstar items".into()), &params.f)
        }
    }
}

#[handler]
pub async fn set_rating(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
    query: Query<SetRatingQuery>,
) -> impl IntoResponse {
    use crate::models::{child, album, artist};
    
    if query.rating < 0 || query.rating > 5 {
        return send_response(SubsonicResponse::new_error(0, "Invalid rating".into()), &params.f);
    }

    // Try child first
    let res = child::Entity::update_many()
        .filter(child::Column::Id.eq(query.id.clone()))
        .col_expr(child::Column::UserRating, Expr::value(query.rating))
        .exec(db.0)
        .await;

    match res {
        Ok(result) if result.rows_affected > 0 => {
            return send_response(SubsonicResponse::new_ok(SubsonicResponseBody::None), &params.f);
        }
        Err(e) => {
            log::error!("Database error in set_rating (child): {}", e);
            return send_response(SubsonicResponse::new_error(0, "Failed to set rating".into()), &params.f);
        }
        _ => {}
    }

    // Try album
    let res = album::Entity::update_many()
        .filter(album::Column::Id.eq(query.id.clone()))
        .col_expr(album::Column::UserRating, Expr::value(query.rating))
        .exec(db.0)
        .await;

    match res {
        Ok(result) if result.rows_affected > 0 => {
            return send_response(SubsonicResponse::new_ok(SubsonicResponseBody::None), &params.f);
        }
        Err(e) => {
            log::error!("Database error in set_rating (album): {}", e);
            return send_response(SubsonicResponse::new_error(0, "Failed to set rating".into()), &params.f);
        }
        _ => {}
    }

    // Try artist
    let res = artist::Entity::update_many()
        .filter(artist::Column::Id.eq(query.id.clone()))
        .col_expr(artist::Column::UserRating, Expr::value(query.rating))
        .exec(db.0)
        .await;

    match res {
        Ok(result) if result.rows_affected > 0 => {
            return send_response(SubsonicResponse::new_ok(SubsonicResponseBody::None), &params.f);
        }
        Err(e) => {
            log::error!("Database error in set_rating (artist): {}", e);
            return send_response(SubsonicResponse::new_error(0, "Failed to set rating".into()), &params.f);
        }
        _ => {}
    }

    send_response(SubsonicResponse::new_error(70, "Item not found".into()), &params.f)
}

#[handler]
pub async fn scrobble(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
    query: Query<ScrobbleQuery>,
) -> impl IntoResponse {
    use crate::models::{child, now_playing};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};

    let submission = query.submission.unwrap_or(true);
    let username = params.u.clone().unwrap_or_else(|| "guest".to_string());
    let player_id = 1; // Default player ID for now

    if !submission {
        let now = chrono::Utc::now();
        let np = now_playing::ActiveModel {
            username: Set(username),
            player_id: Set(player_id),
            song_id: Set(query.id.clone()),
            player_name: Set(params.c.clone()),
            updated_at: Set(now),
        };

        let result = now_playing::Entity::insert(np)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    now_playing::Column::Username,
                    now_playing::Column::PlayerId,
                ])
                .update_columns([
                    now_playing::Column::SongId,
                    now_playing::Column::PlayerName,
                    now_playing::Column::UpdatedAt,
                ])
                .to_owned(),
            )
            .exec(db.0)
            .await;

        if let Err(e) = result {
            log::error!("Database error in update now playing: {}", e);
        }

        return send_response(
            SubsonicResponse::new_ok(SubsonicResponseBody::None),
            &params.f,
        );
    }

    let now = chrono::Utc::now();
    let res = child::Entity::update_many()
        .filter(child::Column::Id.eq(query.id.clone()))
        .col_expr(child::Column::PlayCount, Expr::col(child::Column::PlayCount).add(1))
        .col_expr(child::Column::LastPlayed, Expr::value(Some(now)))
        .exec(db.0)
        .await;

    match res {
        Ok(_) => send_response(SubsonicResponse::new_ok(SubsonicResponseBody::None), &params.f),
        Err(e) => {
            log::error!("Database error in scrobble: {}", e);
            send_response(SubsonicResponse::new_error(0, "Failed to scrobble".into()), &params.f)
        }
    }
}
