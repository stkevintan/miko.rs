use crate::models::{
    album, album_artist, album_genre, artist, child, lyrics, song_artist, user_rating, user_star,
};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, FromQueryResult, Iterable, JoinType, QueryFilter,
    QuerySelect, RelationTrait,
};

#[derive(Debug, FromQueryResult, Clone)]
pub struct LyricsWithMetadata {
    pub title: String,
    pub artist: Option<String>,
    pub content: String,
}

#[derive(sea_orm::FromQueryResult)]
pub struct SongPathInfo {
    pub path: String,
    pub music_folder_id: i32,
}

#[derive(sea_orm::FromQueryResult)]
pub struct FolderPathInfo {
    pub path: String,
}

pub fn song_path_info_query() -> sea_orm::Select<child::Entity> {
    child::Entity::find()
        .filter(child::Column::IsDir.eq(false))
        .filter(child::Column::ContentType.starts_with("audio/"))
        .select_only()
        .column(child::Column::Path)
        .column(child::Column::MusicFolderId)
}

pub fn lyrics_with_metadata_query() -> sea_orm::Select<lyrics::Entity> {
    lyrics::Entity::find()
        .join(JoinType::InnerJoin, lyrics::Relation::Child.def())
        .column_as(child::Column::Title, "title")
        .join_rev(
            JoinType::LeftJoin,
            song_artist::Entity::belongs_to(child::Entity)
                .from(song_artist::Column::SongId)
                .to(child::Column::Id)
                .into(),
        )
        .join_rev(
            JoinType::LeftJoin,
            artist::Entity::belongs_to(song_artist::Entity)
                .from(artist::Column::Id)
                .to(song_artist::Column::ArtistId)
                .into(),
        )
        .column_as(Expr::cust("GROUP_CONCAT(DISTINCT artists.name)"), "artist")
        .group_by(lyrics::Column::SongId)
}

pub fn song_with_metadata_query(username: &str) -> sea_orm::Select<child::Entity> {
    let star_user = username.to_string();
    let rating_user = username.to_string();
    child::Entity::find()
        .select_only()
        .columns(child::Column::iter())
        .column_as(user_star::Column::StarredAt, "starred")
        .column_as(
            Expr::col((user_rating::Entity, user_rating::Column::Rating)).if_null(0),
            "user_rating",
        )
        .column_as(Expr::cust("(SELECT GROUP_CONCAT(a.id || '[:]' || a.name) FROM song_artists sa JOIN artists a ON sa.artist_id = a.id WHERE sa.song_id = children.id)"), "artists")
        .column_as(Expr::cust("(SELECT GROUP_CONCAT(a.id || '[:]' || a.name) FROM album_artists aa JOIN artists a ON aa.artist_id = a.id WHERE aa.album_id = children.album_id)"), "album_artists")
        .column_as(Expr::cust("(SELECT GROUP_CONCAT(genre_name) FROM song_genres WHERE song_id = children.id)"), "genre")
        .column_as(Expr::cust("(SELECT name FROM albums WHERE id = children.album_id)"), "album")
        .join_rev(
            JoinType::LeftJoin,
            user_star::Entity::belongs_to(child::Entity)
                .from(user_star::Column::ItemId)
                .to(child::Column::Id)
                .on_condition(move |_left, _right| {
                    Condition::all()
                        .add(user_star::Column::ItemType.eq("song"))
                        .add(user_star::Column::Username.eq(star_user.clone()))
                })
                .into(),
        )
        .join_rev(
            JoinType::LeftJoin,
            user_rating::Entity::belongs_to(child::Entity)
                .from(user_rating::Column::ItemId)
                .to(child::Column::Id)
                .on_condition(move |_left, _right| {
                    Condition::all()
                        .add(user_rating::Column::ItemType.eq("song"))
                        .add(user_rating::Column::Username.eq(rating_user.clone()))
                })
                .into(),
        )
}

pub fn artist_with_stats_query(username: &str) -> sea_orm::Select<artist::Entity> {
    let star_user = username.to_string();
    let rating_user = username.to_string();
    artist::Entity::find()
        .select_only()
        .columns(artist::Column::iter())
        .column_as(user_star::Column::StarredAt, "starred")
        .column_as(
            Expr::col((user_rating::Entity, user_rating::Column::Rating)).if_null(0),
            "user_rating",
        )
        .column_as(Expr::cust("(SELECT COUNT(DISTINCT album_id) FROM (SELECT album_id FROM children JOIN song_artists ON song_artists.song_id = children.id WHERE song_artists.artist_id = artists.id UNION SELECT album_id FROM album_artists WHERE album_artists.artist_id = artists.id))"), "album_count")
        .join_rev(
            JoinType::LeftJoin,
            user_star::Entity::belongs_to(artist::Entity)
                .from(user_star::Column::ItemId)
                .to(artist::Column::Id)
                .on_condition(move |_left, _right| {
                    Condition::all()
                        .add(user_star::Column::ItemType.eq("artist"))
                        .add(user_star::Column::Username.eq(star_user.clone()))
                })
                .into(),
        )
        .join_rev(
            JoinType::LeftJoin,
            user_rating::Entity::belongs_to(artist::Entity)
                .from(user_rating::Column::ItemId)
                .to(artist::Column::Id)
                .on_condition(move |_left, _right| {
                    Condition::all()
                        .add(user_rating::Column::ItemType.eq("artist"))
                        .add(user_rating::Column::Username.eq(rating_user.clone()))
                })
                .into(),
        )
}

pub fn album_with_stats_query(username: &str) -> sea_orm::Select<album::Entity> {
    let star_user = username.to_string();
    let rating_user = username.to_string();
    album::Entity::find()
        .select_only()
        .columns(album::Column::iter())
        .column_as(
            Expr::col((user_star::Entity, user_star::Column::StarredAt)).max(),
            "starred",
        )
        .column_as(
            Expr::expr(Expr::col((user_rating::Entity, user_rating::Column::Rating)).max())
                .if_null(0),
            "user_rating",
        )
        .column_as(child::Column::Id.count(), "song_count")
        .column_as(Expr::cust("COALESCE(SUM(duration), 0)"), "duration")
        .column_as(Expr::cust("COALESCE(SUM(play_count), 0)"), "play_count")
        .column_as(child::Column::LastPlayed.max(), "last_played")
        .column_as(
            Expr::cust("GROUP_CONCAT(DISTINCT artists.id || '[:]' || artists.name)"),
            "artists",
        )
        .column_as(
            Expr::cust("GROUP_CONCAT(DISTINCT album_genres.genre_name)"),
            "genre",
        )
        .join_rev(
            JoinType::LeftJoin,
            child::Entity::belongs_to(album::Entity)
                .from(child::Column::AlbumId)
                .to(album::Column::Id)
                .into(),
        )
        .join_rev(
            JoinType::LeftJoin,
            album_artist::Entity::belongs_to(album::Entity)
                .from(album_artist::Column::AlbumId)
                .to(album::Column::Id)
                .into(),
        )
        .join_rev(
            JoinType::LeftJoin,
            artist::Entity::belongs_to(album_artist::Entity)
                .from(artist::Column::Id)
                .to(album_artist::Column::ArtistId)
                .into(),
        )
        .join_rev(
            JoinType::LeftJoin,
            album_genre::Entity::belongs_to(album::Entity)
                .from(album_genre::Column::AlbumId)
                .to(album::Column::Id)
                .into(),
        )
        .join_rev(
            JoinType::LeftJoin,
            user_star::Entity::belongs_to(album::Entity)
                .from(user_star::Column::ItemId)
                .to(album::Column::Id)
                .on_condition(move |_left, _right| {
                    Condition::all()
                        .add(user_star::Column::ItemType.eq("album"))
                        .add(user_star::Column::Username.eq(star_user.clone()))
                })
                .into(),
        )
        .join_rev(
            JoinType::LeftJoin,
            user_rating::Entity::belongs_to(album::Entity)
                .from(user_rating::Column::ItemId)
                .to(album::Column::Id)
                .on_condition(move |_left, _right| {
                    Condition::all()
                        .add(user_rating::Column::ItemType.eq("album"))
                        .add(user_rating::Column::Username.eq(rating_user.clone()))
                })
                .into(),
        )
        .group_by(album::Column::Id)
}
