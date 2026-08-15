use crate::{KINDLE_H, KINDLE_W, music::MusicUpdate, weather::WeatherUpdate};
use anyrender::{PaintScene, render_to_buffer};
use anyrender_vello::VelloImageRenderer;
use askama::Template;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use blitz_dom::{DocumentConfig, util::Color};
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::shell::{ColorScheme, Viewport};
use peniko::{Fill, kurbo::Rect};
use std::error::Error;

pub struct Fonts {
    plex_sans_var: String,
    plex_sans_var_italic: String,
}

impl Fonts {
    pub fn load() -> Self {
        const PLEX_SANS_VAR: &[u8] =
            include_bytes!("../templates/assets/IBM Plex Sans Var-Roman.woff2");
        const PLEX_SANS_VAR_ITALIC: &[u8] =
            include_bytes!("../templates/assets/IBM Plex Sans Var-Italic.woff2");

        Self {
            plex_sans_var: BASE64_STANDARD.encode(PLEX_SANS_VAR),
            plex_sans_var_italic: BASE64_STANDARD.encode(PLEX_SANS_VAR_ITALIC),
        }
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct KiiinTemplate {
    pub music: Option<MusicUpdate>,
    pub weather: Option<WeatherUpdate>,
    pub fonts: Fonts,
}

impl KiiinTemplate {
    pub fn render_rgba(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let rendered = self.render()?;
        let mut document = HtmlDocument::from_html(
            &rendered,
            DocumentConfig {
                viewport: Some(Viewport::new(
                    KINDLE_H as u32,
                    KINDLE_W as u32,
                    1.0,
                    ColorScheme::Light,
                )),
                ..Default::default()
            },
        );

        document.as_mut().resolve(0.0);

        let buffer = render_to_buffer::<VelloImageRenderer, _>(
            |scene| {
                scene.fill(
                    Fill::NonZero,
                    Default::default(),
                    Color::WHITE,
                    Default::default(),
                    &Rect::new(0.0, 0.0, KINDLE_W as f64, KINDLE_H as f64),
                );
                paint_scene(
                    scene,
                    document.as_mut(),
                    1.0,
                    KINDLE_W as u32,
                    KINDLE_H as u32,
                );
            },
            KINDLE_W as u32,
            KINDLE_H as u32,
        );

        Ok(buffer)
    }
}
