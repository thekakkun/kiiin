extern crate mpd;

mod music;
mod template;
mod weather;

use crate::music::monitor_mpd;
use crate::music::{AlbumArt, Song};
use crate::template::{KiiinTemplate, MusicTemplate};

use image::{ImageReader, imageops::rotate90};
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use std::error::Error;
use tempfile::NamedTempFile;
use tokio::fs::File;
use tokio::sync::mpsc;

pub const KINDLE_H: u16 = 1072;
pub const KINDLE_W: u16 = 1448;

#[derive(Debug)]
enum Event {
    Music(Option<Song>, Box<Option<AlbumArt>>, Option<Song>),
    Weather,
}

// Processes image in path for Kindle
fn process_img(img: &NamedTempFile) {
    let path = img.path();
    let img = ImageReader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .into_luma8();
    let rotated = rotate90(&img);
    rotated.save(path).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = mpsc::channel(100);

    tokio::task::spawn_blocking(|| {
        if let Err(e) = monitor_mpd(tx) {
            eprintln!("MPD error: {}", e);
        };
    });

    let mut template = KiiinTemplate { music: None };
    let mut refresh = false;

    while let Some(event) = rx.recv().await {
        match event {
            Event::Music(ref current_song, ref _album_art, ref _next_song) => {
                refresh = matches!(
                    (&template.music, &current_song),
                    (
                        Some(MusicTemplate { current_song: Some(previous), .. }),
                        Some(current)
                    ) if previous.album != current.album
                );

                template.music = Some(event.try_into()?);
            }
            Event::Weather => println!("Got from weather"),
        }

        let dash_img = template.screenshot()?;
        process_img(&dash_img);

        let file_part = Part::stream(File::from(dash_img.into_file()))
            .file_name("photo.png")
            .mime_str("image/png")?;
        let bool_part = Part::text(refresh.to_string());

        let form = Form::new()
            .part("file", file_part)
            .part("refresh", bool_part);
        let client = Client::new();
        client
            .post("http://kindle.lan:3000/image")
            .multipart(form)
            .send()
            .await?;
    }

    Ok(())
}
