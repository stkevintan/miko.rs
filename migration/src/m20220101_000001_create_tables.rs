use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::async_trait::async_trait;

#[derive(Iden)]
pub enum Users {
    #[iden = "users"]
    Table,
    Username,
    Password,
    Email,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    ScrobblingEnabled,
    MaxBitRate,
    SettingsRole,
    DownloadRole,
    UploadRole,
    AdminRole,
    PlaylistRole,
    CoverArtRole,
    CommentRole,
    PodcastRole,
    StreamRole,
    JukeboxRole,
    ShareRole,
    VideoConversionRole,
    AvatarLastChanged,
}

#[derive(Iden)]
pub enum MusicFolders {
    #[iden = "music_folders"]
    Table,
    Id,
    Path,
    Name,
}

#[derive(Iden)]
pub enum UserMusicFolders {
    #[iden = "user_music_folders"]
    Table,
    Username,
    MusicFolderId,
}

#[derive(Iden)]
enum Children {
    #[iden = "children"]
    Table,
    Id,
    Parent,
    IsDir,
    Title,
    Track,
    Year,
    Size,
    ContentType,
    Suffix,
    TranscodedContentType,
    TranscodedSuffix,
    Duration,
    BitRate,
    Path,
    IsVideo,
    UserRating,
    AverageRating,
    PlayCount,
    LastPlayed,
    DiscNumber,
    Created,
    Starred,
    AlbumId,
    MusicFolderId,
    #[iden = "type"]
    Type,
}

#[derive(Iden)]
enum Artists {
    #[iden = "artists"]
    Table,
    Id,
    Name,
    ArtistImageUrl,
    Starred,
    UserRating,
    AverageRating,
}

#[derive(Iden)]
enum Albums {
    #[iden = "albums"]
    Table,
    Id,
    Name,
    Created,
    Starred,
    UserRating,
    AverageRating,
    Year,
}

#[derive(Iden)]
enum Genres {
    #[iden = "genres"]
    Table,
    Name,
}

#[derive(Iden)]
enum SongArtists {
    #[iden = "song_artists"]
    Table,
    SongId,
    ArtistId,
}

#[derive(Iden)]
enum SongGenres {
    #[iden = "song_genres"]
    Table,
    SongId,
    GenreName,
}

#[derive(Iden)]
enum AlbumArtists {
    #[iden = "album_artists"]
    Table,
    AlbumId,
    ArtistId,
}

#[derive(Iden)]
enum Playlists {
    #[iden = "playlists"]
    Table,
    Id,
    Name,
    Comment,
    Owner,
    Public,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum PlaylistSongs {
    #[iden = "playlist_songs"]
    Table,
    PlaylistId,
    SongId,
    Index,
}

#[derive(Iden)]
enum Lyrics {
    #[iden = "lyrics"]
    Table,
    SongId,
    Content,
}

#[derive(Iden)]
enum AlbumGenres {
    #[iden = "album_genres"]
    Table,
    AlbumId,
    GenreName,
}

#[derive(Iden)]
enum NowPlaying {
    #[iden = "now_playing"]
    Table,
    Username,
    PlayerName,
    SongId,
    UpdatedAt,
}

#[derive(Iden)]
enum Bookmark {
    #[iden = "bookmark"]
    Table,
    Username,
    SongId,
    Position,
    Comment,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum PlayQueue {
    #[iden = "play_queue"]
    Table,
    Username,
    Current,
    Position,
    Changed,
    ChangedBy,
}

#[derive(Iden)]
enum PlayQueueSong {
    #[iden = "play_queue_song"]
    Table,
    Username,
    Index,
    SongId,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Username).string().primary_key())
                    .col(ColumnDef::new(Users::Password).string().not_null())
                    .col(ColumnDef::new(Users::Email).string())
                    .col(ColumnDef::new(Users::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Users::UpdatedAt).date_time().not_null())
                    .col(ColumnDef::new(Users::DeletedAt).date_time())
                    .col(ColumnDef::new(Users::ScrobblingEnabled).boolean().not_null().default(true))
                    .col(ColumnDef::new(Users::MaxBitRate).integer())
                    .col(ColumnDef::new(Users::SettingsRole).boolean().not_null().default(false))
                    .col(ColumnDef::new(Users::DownloadRole).boolean().not_null().default(true))
                    .col(ColumnDef::new(Users::UploadRole).boolean().not_null().default(false))
                    .col(ColumnDef::new(Users::AdminRole).boolean().not_null().default(false))
                    .col(ColumnDef::new(Users::PlaylistRole).boolean().not_null().default(true))
                    .col(ColumnDef::new(Users::CoverArtRole).boolean().not_null().default(true))
                    .col(ColumnDef::new(Users::CommentRole).boolean().not_null().default(true))
                    .col(ColumnDef::new(Users::PodcastRole).boolean().not_null().default(false))
                    .col(ColumnDef::new(Users::StreamRole).boolean().not_null().default(true))
                    .col(ColumnDef::new(Users::JukeboxRole).boolean().not_null().default(false))
                    .col(ColumnDef::new(Users::ShareRole).boolean().not_null().default(true))
                    .col(ColumnDef::new(Users::VideoConversionRole).boolean().not_null().default(false))
                    .col(ColumnDef::new(Users::AvatarLastChanged).date_time())
                    .to_owned(),
            )
            .await?;

        // MusicFolders
        manager
            .create_table(
                Table::create()
                    .table(MusicFolders::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(MusicFolders::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(MusicFolders::Path).string().not_null().unique_key())
                    .col(ColumnDef::new(MusicFolders::Name).string())
                    .to_owned(),
            )
            .await?;

        // UserMusicFolders
        manager
            .create_table(
                Table::create()
                    .table(UserMusicFolders::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserMusicFolders::Username).string().not_null())
                    .col(ColumnDef::new(UserMusicFolders::MusicFolderId).integer().not_null())
                    .primary_key(Index::create().col(UserMusicFolders::Username).col(UserMusicFolders::MusicFolderId))
                    .to_owned(),
            )
            .await?;

        // Children (Songs/Directories)
        manager
            .create_table(
                Table::create()
                    .table(Children::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Children::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Children::Parent).string())
                    .col(ColumnDef::new(Children::IsDir).boolean().not_null())
                    .col(ColumnDef::new(Children::Title).string().not_null())
                    .col(ColumnDef::new(Children::Track).integer().not_null().default(0))
                    .col(ColumnDef::new(Children::Year).integer().not_null().default(0))
                    .col(ColumnDef::new(Children::Size).big_integer().not_null().default(0))
                    .col(ColumnDef::new(Children::ContentType).string())
                    .col(ColumnDef::new(Children::Suffix).string())
                    .col(ColumnDef::new(Children::TranscodedContentType).string())
                    .col(ColumnDef::new(Children::TranscodedSuffix).string())
                    .col(ColumnDef::new(Children::Duration).integer().not_null().default(0))
                    .col(ColumnDef::new(Children::BitRate).integer().not_null().default(0))
                    .col(ColumnDef::new(Children::Path).string().not_null().unique_key())
                    .col(ColumnDef::new(Children::IsVideo).boolean().not_null().default(false))
                    .col(ColumnDef::new(Children::UserRating).integer().not_null().default(0))
                    .col(ColumnDef::new(Children::AverageRating).double().not_null().default(0.0))
                    .col(ColumnDef::new(Children::PlayCount).big_integer().not_null().default(0))
                    .col(ColumnDef::new(Children::LastPlayed).date_time())
                    .col(ColumnDef::new(Children::DiscNumber).integer().not_null().default(0))
                    .col(ColumnDef::new(Children::Created).date_time())
                    .col(ColumnDef::new(Children::Starred).date_time())
                    .col(ColumnDef::new(Children::AlbumId).string())
                    .col(ColumnDef::new(Children::MusicFolderId).integer().not_null())
                    .col(ColumnDef::new(Children::Type).string().not_null().default("music"))
                    .to_owned(),
            )
            .await?;

        // Artists
        manager
            .create_table(
                Table::create()
                    .table(Artists::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Artists::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Artists::Name).string().not_null())
                    .col(ColumnDef::new(Artists::ArtistImageUrl).string())
                    .col(ColumnDef::new(Artists::Starred).date_time())
                    .col(ColumnDef::new(Artists::UserRating).integer().not_null().default(0))
                    .col(ColumnDef::new(Artists::AverageRating).double().not_null().default(0.0))
                    .to_owned(),
            )
            .await?;

        // Albums
        manager
            .create_table(
                Table::create()
                    .table(Albums::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Albums::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Albums::Name).string().not_null())
                    .col(ColumnDef::new(Albums::Created).date_time().not_null())
                    .col(ColumnDef::new(Albums::Starred).date_time())
                    .col(ColumnDef::new(Albums::UserRating).integer().not_null().default(0))
                    .col(ColumnDef::new(Albums::AverageRating).double().not_null().default(0.0))
                    .col(ColumnDef::new(Albums::Year).integer().not_null().default(0))
                    .to_owned(),
            )
            .await?;

        // Genres
        manager
            .create_table(
                Table::create()
                    .table(Genres::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Genres::Name).string().not_null().primary_key())
                    .to_owned(),
            )
            .await?;

        // SongArtists
        manager
            .create_table(
                Table::create()
                    .table(SongArtists::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SongArtists::SongId).string().not_null())
                    .col(ColumnDef::new(SongArtists::ArtistId).string().not_null())
                    .primary_key(Index::create().col(SongArtists::SongId).col(SongArtists::ArtistId))
                    .to_owned(),
            )
            .await?;

        // SongGenres
        manager
            .create_table(
                Table::create()
                    .table(SongGenres::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SongGenres::SongId).string().not_null())
                    .col(ColumnDef::new(SongGenres::GenreName).string().not_null())
                    .primary_key(Index::create().col(SongGenres::SongId).col(SongGenres::GenreName))
                    .to_owned(),
            )
            .await?;

        // AlbumArtists
        manager
            .create_table(
                Table::create()
                    .table(AlbumArtists::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AlbumArtists::AlbumId).string().not_null())
                    .col(ColumnDef::new(AlbumArtists::ArtistId).string().not_null())
                    .primary_key(Index::create().col(AlbumArtists::AlbumId).col(AlbumArtists::ArtistId))
                    .to_owned(),
            )
            .await?;

        // Playlists
        manager
            .create_table(
                Table::create()
                    .table(Playlists::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Playlists::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Playlists::Name).string().not_null())
                    .col(ColumnDef::new(Playlists::Comment).string())
                    .col(ColumnDef::new(Playlists::Owner).string().not_null())
                    .col(ColumnDef::new(Playlists::Public).boolean().not_null().default(false))
                    .col(ColumnDef::new(Playlists::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Playlists::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        // PlaylistSongs
        manager
            .create_table(
                Table::create()
                    .table(PlaylistSongs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PlaylistSongs::PlaylistId).integer().not_null())
                    .col(ColumnDef::new(PlaylistSongs::SongId).string().not_null())
                    .col(ColumnDef::new(PlaylistSongs::Index).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(PlaylistSongs::PlaylistId)
                            .col(PlaylistSongs::SongId),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("idx-playlist-song-index-unique")
                            .col(PlaylistSongs::PlaylistId)
                            .col(PlaylistSongs::Index),
                    )
                    .to_owned(),
            )
            .await?;

        // Lyrics
        manager
            .create_table(
                Table::create()
                    .table(Lyrics::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Lyrics::SongId).string().not_null().primary_key())
                    .col(ColumnDef::new(Lyrics::Content).text().not_null())
                    .to_owned(),
            )
            .await?;

        // AlbumGenres
        manager
            .create_table(
                Table::create()
                    .table(AlbumGenres::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AlbumGenres::AlbumId).string().not_null())
                    .col(ColumnDef::new(AlbumGenres::GenreName).string().not_null())
                    .primary_key(Index::create().col(AlbumGenres::AlbumId).col(AlbumGenres::GenreName))
                    .to_owned(),
            )
            .await?;

        // NowPlaying
        manager
            .create_table(
                Table::create()
                    .table(NowPlaying::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(NowPlaying::Username).string().not_null())
                    .col(ColumnDef::new(NowPlaying::PlayerName).string().not_null())
                    .col(ColumnDef::new(NowPlaying::SongId).string().not_null())
                    .col(ColumnDef::new(NowPlaying::UpdatedAt).date_time().not_null())
                    .primary_key(Index::create().col(NowPlaying::Username).col(NowPlaying::PlayerName))
                    .to_owned(),
            )
            .await?;

        // Bookmark
        manager
            .create_table(
                Table::create()
                    .table(Bookmark::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Bookmark::Username).string().not_null())
                    .col(ColumnDef::new(Bookmark::SongId).string().not_null())
                    .col(ColumnDef::new(Bookmark::Position).big_integer().not_null())
                    .col(ColumnDef::new(Bookmark::Comment).string())
                    .col(ColumnDef::new(Bookmark::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Bookmark::UpdatedAt).date_time().not_null())
                    .primary_key(Index::create().col(Bookmark::Username).col(Bookmark::SongId))
                    .to_owned(),
            )
            .await?;

        // PlayQueue
        manager
            .create_table(
                Table::create()
                    .table(PlayQueue::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PlayQueue::Username).string().not_null().primary_key())
                    .col(ColumnDef::new(PlayQueue::Current).string())
                    .col(ColumnDef::new(PlayQueue::Position).big_integer().not_null().default(0))
                    .col(ColumnDef::new(PlayQueue::Changed).date_time().not_null())
                    .col(ColumnDef::new(PlayQueue::ChangedBy).string().not_null())
                    .to_owned(),
            )
            .await?;

        // PlayQueueSong
        manager
            .create_table(
                Table::create()
                    .table(PlayQueueSong::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PlayQueueSong::Username).string().not_null())
                    .col(ColumnDef::new(PlayQueueSong::Index).integer().not_null())
                    .col(ColumnDef::new(PlayQueueSong::SongId).string().not_null())
                    .primary_key(Index::create().col(PlayQueueSong::Username).col(PlayQueueSong::Index))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(PlayQueueSong::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(PlayQueue::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Bookmark::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(NowPlaying::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(AlbumGenres::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Lyrics::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(PlaylistSongs::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Playlists::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(AlbumArtists::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(SongGenres::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(SongArtists::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Genres::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Albums::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Artists::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Children::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(UserMusicFolders::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(MusicFolders::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Users::Table).to_owned()).await?;
        Ok(())
    }
}
