use crate::Event;
use mpd::{Client, Idle};

use std::env;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub date: String,
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
        }
    }
}

pub type AlbumArt = Vec<u8>;

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

pub fn mpd(tx: mpsc::Sender<Event>) -> Result<(), &'static str> {
    let mut client = init_mpd()?;
    loop {
        if let Ok(_) = client.wait(&[mpd::Subsystem::Player]) {
            let status = client.status().map_err(|_| "Could not get status")?;

            let mut current_song = None;
            let mut album_art = None;
            let mut next_song = None;

            if let Some(queue_place) = status.song {
                current_song = client
                    .playlistid(queue_place.id)
                    .map_err(|_| "Could not get current song")?;

                if let Some(ref song) = current_song {
                    album_art = Some(
                        client
                            .albumart(&song)
                            .map_err(|_| "Could not get album art")?,
                    );
                }
            }

            if let Some(queue_place) = status.nextsong {
                next_song = client
                    .playlistid(queue_place.id)
                    .map_err(|_| "Could not get next song")?;
            }

            let _ = tx.blocking_send(Event::Music(
                current_song.map(Song::from),
                album_art,
                next_song.map(Song::from),
            ));
        }
    }
}
