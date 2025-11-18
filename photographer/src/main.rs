extern crate mpd;

mod music;
mod template;
mod weather;

use crate::music::{SongChange, monitor_mpd};
use crate::template::{KiiinTemplate, MusicTemplate};
use crate::weather::{WeatherUpdate, monitor_weather};

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
    Music(SongChange),
    Weather(WeatherUpdate),
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

    let tx_weather = tx.clone();
    tokio::task::spawn_blocking(|| {
        if let Err(e) = monitor_mpd(tx) {
            eprintln!("MPD error: {}", e);
        };
    });
    tokio::task::spawn(async move {
        if let Err(e) = monitor_weather(tx_weather).await {
            eprintln!("Weather error: {}", e);
        }
    });

    let mut template = KiiinTemplate {
        music: None,
        weather: None,
    };

    while let Some(event) = rx.recv().await {
        let refresh = match event {
            Event::Music(SongChange {
                ref current_song, ..
            }) => {
                println!("SongChange");
                let refresh = matches!(
                    (&template.music, &current_song),
                    (
                        Some(MusicTemplate { current_song: Some(previous), .. }),
                        Some(current)
                    ) if previous.album != current.album
                );

                template.music = Some(event.try_into()?);
                refresh
            }
            Event::Weather(weather_update) => {
                println!("{:?}", weather_update);
                template.weather = Some(weather_update);
                true
            }
        };

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
