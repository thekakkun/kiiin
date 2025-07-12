extern crate mpd;

use mpd::Client;
use std::env;

fn init_mpd() -> Client {
    let mpd_host;
    let mpd_pass;

    let host_pass = env::var("MPD_HOST").unwrap_or("localhost".to_string());
    let binding = host_pass.rsplitn(2, "@").collect::<Vec<_>>();
    match binding.as_slice() {
        [host] => {
            mpd_pass = None;
            mpd_host = host;
        }
        [host, pass] => {
            mpd_pass = Some(pass);
            mpd_host = host;
        }
        _ => unreachable!(),
    };
    let mpd_port = env::var("MPD_PORT").unwrap_or("6600".to_string());

    let mut conn = Client::connect(format!("{mpd_host}:{mpd_port}")).unwrap();
    if let Some(password) = mpd_pass {
        let _ = conn.login(password);
    }

    conn
}
fn main() {
    let mut client = init_mpd();
    let song = client.currentsong().unwrap();
    if let Some(song) = song {
        println!("{:?}", song)
    }
}
