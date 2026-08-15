extern crate mpd;

mod music;
mod template;
mod weather;

use crate::music::{MusicUpdate, monitor_mpd};
use crate::template::{Fonts, KiiinTemplate};
use crate::weather::{WeatherUpdate, monitor_weather};

use image::imageops::rotate90;
use image::{ImageBuffer, Rgba};
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use std::error::Error;
use std::io::Cursor;
use tokio::sync::mpsc;

pub const KINDLE_H: u16 = 1072;
pub const KINDLE_W: u16 = 1448;

#[derive(Debug)]
enum Event {
    Music(MusicUpdate),
    Weather(WeatherUpdate),
}

fn rgba_buffer_to_png_bytes(raw: Vec<u8>) -> Result<Vec<u8>, image::ImageError> {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(KINDLE_W as u32, KINDLE_H as u32, raw)
            .expect("buffer size doesn't match width*height*4");

    let mut png_bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)?;

    Ok(png_bytes)
}

fn process_img_bytes(png_bytes: &[u8]) -> Result<Vec<u8>, image::ImageError> {
    let img = image::load_from_memory(png_bytes)?.into_luma8();
    let rotated = rotate90(&img);

    let mut out_bytes: Vec<u8> = Vec::new();
    rotated.write_to(&mut Cursor::new(&mut out_bytes), image::ImageFormat::Png)?;
    Ok(out_bytes)
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
        fonts: Fonts::load(),
        music: None,
        weather: None,
    };

    while let Some(event) = rx.recv().await {
        let mut refresh = false;
        match event {
            Event::Music(music_update) => {
                let previous_album = template
                    .music
                    .as_ref()
                    .and_then(|m| m.current_song.as_ref().map(|s| &s.album));
                let next_album = music_update.current_song.as_ref().map(|s| &s.album);
                refresh = previous_album != next_album;

                template.music = Some(music_update);
            }
            Event::Weather(weather_update) => {
                template.weather = Some(weather_update);
            }
        };

        let dash_rgba = template.render_rgba()?;
        let dash_png = rgba_buffer_to_png_bytes(dash_rgba)?;
        std::fs::write("foo.png", &dash_png)?;
        println!("Wrote debug image");
        let dash_img = process_img_bytes(&dash_png)?;

        // let dash_img = template.screenshot()?;
        // process_img(&dash_img);

        let img_part = Part::bytes(dash_img)
            .file_name("photo.png")
            .mime_str("image/png")?;
        // let file_part = Part::stream(File::from(dash_img.into_file()))
        //     .file_name("photo.png")
        //     .mime_str("image/png")?;
        let bool_part = Part::text(refresh.to_string());

        let form = Form::new()
            .part("file", img_part)
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
