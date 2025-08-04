use crate::music::Song;
use askama::Template;

#[derive(Template)]
#[template(path = "music.html")]
pub struct MusicTemplate<'a> {
    pub current_song: &'a Song,
    pub album_art: &'a str,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub music_html: &'a str,
}
