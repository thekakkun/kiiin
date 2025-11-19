use crate::Event;
use askama::Template;
use base64::{Engine, engine::general_purpose::STANDARD};
use mpd::{Client, Idle};
use std::{env, error::Error};
use tokio::sync::mpsc;

#[derive(Debug, Template, Clone)]
#[template(path = "music.html")]
pub(crate) struct MusicUpdate {
    pub current_song: Option<Song>,
    pub album_art: Option<String>,
    pub next_song: Option<Song>,
}

impl TryFrom<&mut Client> for MusicUpdate {
    type Error = Box<dyn Error>;

    fn try_from(client: &mut Client) -> Result<Self, Self::Error> {
        let status = client.status()?;

        let mut current_song = None;
        let mut album_art = None;
        let mut next_song = None;

        if let Some(queue_place) = status.song {
            current_song = client.playlistid(queue_place.id)?;

            if let Some(ref song) = current_song {
                album_art = match client.albumart(&song).ok() {
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
        }

        if let Some(queue_place) = status.nextsong {
            next_song = client.playlistid(queue_place.id)?;
        }

        Ok(Self {
            current_song: current_song.map(Song::from),
            album_art,
            next_song: next_song.map(Song::from),
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Song {
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
                (name, value) if name == "Album" => album = Some(value),
                (name, value) if name == "Date" => date = Some(value),
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

fn init_mpd() -> Result<Client, Box<dyn Error>> {
    let mpd_host_pass = env::var("MPD_HOST").unwrap_or(String::from("localhost"));
    let mpd_port = env::var("MPD_PORT").unwrap_or(String::from("6600"));

    let mpd_host;
    let mpd_pass;
    if let Some((pass, host)) = mpd_host_pass.split_once('@') {
        mpd_host = host;
        mpd_pass = Some(pass);
    } else {
        mpd_host = &mpd_host_pass;
        mpd_pass = None;
    }

    let mut client = Client::connect(format!("{mpd_host}:{mpd_port}"))?;
    if let Some(password) = mpd_pass {
        client.login(password)?;
    }

    Ok(client)
}

pub(crate) fn monitor_mpd(tx: mpsc::Sender<Event>) -> Result<(), Box<dyn Error>> {
    let mut client = init_mpd()?;
    let _ = tx.blocking_send(Event::Music((&mut client).try_into()?));

    loop {
        if client
            .wait(&[mpd::Subsystem::Player, mpd::Subsystem::Playlist])
            .is_ok()
        {
            let _ = tx.blocking_send(Event::Music((&mut client).try_into()?));
        }
    }
}
