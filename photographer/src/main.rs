extern crate mpd;

mod music;
mod template;
mod weather;

use crate::music::monitor_mpd;
use crate::music::{AlbumArt, Song};
use crate::template::{DashTemplate, MusicTemplate, generate_uri};
use crate::weather::monitor_weather;

use askama::Template;
use image::{ImageReader, imageops::rotate90};
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use std::env;
use std::error::Error;
use std::io::Write;
use std::process::Command;
use tempfile::{Builder, NamedTempFile};
use tokio::fs::File;
use tokio::sync::mpsc;

const KINDLE_H: u16 = 1072;
const KINDLE_W: u16 = 1448;

#[derive(Debug)]
enum Event {
    Music(Option<Song>, Box<Option<AlbumArt>>, Option<Song>),
    Weather,
}

fn html_to_screenshot(html: String) -> Result<NamedTempFile, Box<dyn Error>> {
    let profile = env::var("FIREFOX_PROFILE").ok();

    let mut dash_file = NamedTempFile::new()?;
    write!(dash_file, "{}", html)?;

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
    let tx2 = tx.clone();

    tokio::task::spawn_blocking(|| {
        if let Err(e) = monitor_mpd(tx) {
            eprintln!("MPD error: {}", e);
        };
    });
    tokio::task::spawn(async move {
        if let Err(e) = monitor_weather(tx2).await {
            eprintln!("Weather monitor error: {}", e);
        }
    });

    let mut song: Option<Song> = None;
    let mut music_html = String::default();
    let mut refresh = false;

    while let Some(event) = rx.recv().await {
        match event {
            Event::Music(current_song, album_art, next_song) => {
                let data_uri = generate_uri(*album_art);

                music_html = MusicTemplate {
                    current_song: &current_song,
                    album_art: &data_uri,
                    next_song: &next_song,
                }
                .render()?;

                if let (Some(song), Some(current_song)) = (&song, &current_song) {
                    refresh = song.album != current_song.album
                }
                song = current_song;
            }
            Event::Weather => println!("Got from weather"),
        }

        let dash_rendered = DashTemplate {
            music_html: &music_html,
        }
        .render()?;

        let dash_img = html_to_screenshot(dash_rendered)?;
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
