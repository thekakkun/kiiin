use crate::{KINDLE_H, KINDLE_W, music::MusicUpdate, weather::WeatherUpdate};
use askama::Template;
use std::io::Write;

use std::{env, error::Error, process::Command};
use tempfile::{Builder, NamedTempFile};

#[derive(Template)]
#[template(path = "index.html")]
pub struct KiiinTemplate {
    pub music: Option<MusicUpdate>,
    pub weather: Option<WeatherUpdate>,
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
