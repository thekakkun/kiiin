extern crate mpd;

mod music;
mod template;

use crate::music::mpd;
use crate::music::{AlbumArt, Song};
use crate::template::{IndexTemplate, MusicTemplate};

use base64::engine::general_purpose::STANDARD;

use askama::Template;
use base64::Engine;
use image::ImageReader;
use image::imageops::rotate90;
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::{Builder, NamedTempFile};
use tokio::fs::File;
use tokio::sync::mpsc;

const KINDLE_H: u16 = 1072;
const KINDLE_W: u16 = 1448;

#[derive(Debug)]
enum Event {
    Music(Option<Song>, Option<AlbumArt>, Option<Song>),
    Weather,
}

fn process_img(path: &Path) {
    let img = ImageReader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .into_luma8();
    let rotated = rotate90(&img);
    rotated.save(path).unwrap();
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(100);

    tokio::task::spawn_blocking(|| {
        if let Err(e) = mpd(tx) {
            eprintln!("MPD error: {}", e);
        };
    });

    let mut music_html = String::default();

    while let Some(event) = rx.recv().await {
        match event {
            Event::Music(current_song, album_art, next_song) => {
                println!("{:?}", current_song);
                println!("{:?}", next_song);

                let mime;
                let data_uri;

                if let Some(art) = album_art {
                    mime = infer::get(&art)
                        .map(|t| t.mime_type())
                        .unwrap_or("image/png");
                    let base64_data = STANDARD.encode(art);
                    data_uri = format!("data:{};base64,{}", mime, base64_data);
                } else {
                    data_uri = String::from("foo");
                }

                music_html = MusicTemplate {
                    current_song: &current_song.unwrap(),
                    album_art: &data_uri,
                }
                .render()
                .unwrap();
            }
            Event::Weather => println!("Got from weather"),
        }

        let dash_rendered = IndexTemplate {
            music_html: &music_html,
        }
        .render()
        .unwrap();
        let mut dash_file = NamedTempFile::new().unwrap();
        write!(dash_file, "{}", dash_rendered);

        let mut dash_img = Builder::new().suffix(".png").tempfile().unwrap();

        let _ = Command::new("firefox")
            .args([
                "--headless",
                "-P",
                "screenshot",
                "--screenshot",
                dash_img.path().to_str().unwrap(),
                "--window-size",
                &format!("{},{}", KINDLE_W, KINDLE_H),
                &format!("file:///{}", dash_file.path().display()),
            ])
            .status();

        process_img(dash_img.path());

        let file = File::open(dash_img.path()).await.unwrap();
        let file_part = Part::stream(file)
            .file_name("dash.png")
            .mime_str("image/png")
            .unwrap();

        let form = Form::new().part("file", file_part);
        let client = Client::new();
        let res = client
            .post("http://kindle.lan:3000/image")
            .multipart(form)
            .send()
            .await
            .unwrap();
    }
}
