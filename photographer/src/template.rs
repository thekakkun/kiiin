use crate::music::{AlbumArt, Song};
use askama::Template;

use base64::{Engine, engine::general_purpose::STANDARD};
#[derive(Template)]
#[template(path = "music.html")]
pub struct MusicTemplate<'a> {
    pub current_song: &'a Option<Song>,
    pub album_art: &'a Option<String>,
    pub next_song: &'a Option<Song>,
}

pub fn generate_uri(album_art: Option<AlbumArt>) -> Option<String> {
    match album_art {
        Some(art) => {
            let mime = infer::get(&art)
                .map(|t| t.mime_type())
                .unwrap_or("image/png");
            let base64_data = STANDARD.encode(art);
            Some(format!("data:{};base64,{}", mime, base64_data))
        }
        None => None,
    }
}

#[derive(Template)]
#[template(path = "dash.html")]
pub struct DashTemplate<'a> {
    pub music_html: &'a str,
}
