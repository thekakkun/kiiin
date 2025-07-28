extern crate mpd;

use mpd::{Client, Idle};
use std::env;
use tokio::sync::mpsc;

enum Event {
    MPD(Song),
    Weather,
}

struct Song {
    title: String,
    artist: String,
    album: String,
    date: String,
    album_art: Option<Vec<u8>>,
}
impl From<mpd::Song> for Song {
    fn from(value: mpd::Song) -> Self {
        let mut album = None;
        let mut date = None;
        for tag in value.tags.into_iter() {
            match tag {
                (name, value) if name == String::from("Album") => album = Some(value),
                (name, value) if name == String::from("Date") => date = Some(value),
                _ => {}
            }
        }
        Song {
            title: value.title.unwrap_or(String::from("Title not found")),
            artist: value.artist.unwrap_or(String::from("Artist not found")),
            album: album.unwrap_or(String::from("Title not found")),
            date: date.unwrap_or(String::from("Title not found")),
            album_art: None,
        }
    }
}

fn init_mpd() -> Result<Client, &'static str> {
    let mpd_host_pass = env::var("MPD_HOST").unwrap_or(String::from("localhost"));
    let mpd_port = env::var("MPD_PORT").unwrap_or(String::from("6600"));
    let mpd_host: &str;
    let mpd_pass: Option<&str>;

    if let Some((pass, host)) = mpd_host_pass.split_once('@') {
        mpd_host = host;
        mpd_pass = Some(pass);
    } else {
        mpd_host = &mpd_host_pass;
        mpd_pass = None;
    }
    let mut client = Client::connect(format!("{mpd_host}:{mpd_port}"))
        .map_err(|_| "Error connecting to client")?;
    if let Some(password) = mpd_pass {
        client
            .login(password)
            .map_err(|_| "Could not log in to client")?;
    }

    Ok(client)
}

fn mpd(tx: mpsc::Sender<Event>) -> Result<(), &'static str> {
    let mut client = init_mpd()?;
    loop {
        if let Ok(_) = client.wait(&[mpd::Subsystem::Player]) {
            println!("{:?}", client.currentsong())
        }
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(100);

    tokio::task::spawn_blocking(move || {
        mpd(tx);
    });

    while let Some(event) = rx.recv().await {
        match event {
            Event::MPD(song) => println!("Got from MPD"),
            Event::Weather => println!("Got from weather"),
        }
    }
}
