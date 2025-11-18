use crate::{
    Event, KINDLE_H, KINDLE_W,
    music::{AlbumArt, Song, SongChange},
    weather::WeatherUpdate,
};
use askama::Template;
use std::io::Write;

use base64::{Engine, engine::general_purpose::STANDARD};
use std::{env, error::Error, process::Command};
use tempfile::{Builder, NamedTempFile};

#[derive(Template)]
#[template(path = "music.html")]
pub struct MusicTemplate {
    pub current_song: Option<Song>,
    pub album_art: Option<String>,
    pub next_song: Option<Song>,
}

impl TryFrom<Event> for MusicTemplate {
    type Error = askama::Error;

    fn try_from(value: Event) -> Result<Self, Self::Error> {
        if let Event::Music(SongChange {
            current_song,
            album_art,
            next_song,
        }) = value
        {
            let data_uri = generate_uri(*album_art);
            Ok(Self {
                current_song,
                album_art: data_uri,
                next_song,
            })
        } else {
            Err(askama::Error::ValueType)
        }
    }
}

fn generate_uri(album_art: Option<AlbumArt>) -> Option<String> {
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

type WeatherTemplate = WeatherUpdate;

#[derive(Template)]
#[template(path = "index.html")]
pub struct KiiinTemplate {
    pub music: Option<MusicTemplate>,
    pub weather: Option<WeatherTemplate>,
}

impl KiiinTemplate {
    pub fn screenshot(&self) -> Result<NamedTempFile, Box<dyn Error>> {
        let rendered = self.render()?;

        let profile = env::var("FIREFOX_PROFILE").ok();

        let mut dash_file = NamedTempFile::new()?;
        write!(dash_file, "{}", rendered)?;

        let dash_img = Builder::new().suffix(".png").tempfile().unwrap();

        let mut command = Command::new("firefox");
        command.arg("--headless");
        if let Some(p) = profile {
            command.args(["-P", &p]);
        }
        command.args([
            "--screenshot",
            dash_img.path().to_str().unwrap(),
            "--window-size",
            &format!("{},{}", KINDLE_W, KINDLE_H),
        ]);
        command
            .arg(format!("file:///{}", dash_file.path().display()))
            .output()?;

        Ok(dash_img)
    }
}
