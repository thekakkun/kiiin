use chrono::NaiveDateTime;
use futures_lite::stream::StreamExt;
use std::error::Error;
use url::Url;

use lapin::{
    Connection, ConnectionProperties, Consumer, ExchangeKind,
    options::{
        BasicAckOptions, BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
};
use tokio::sync::mpsc;

use crate::Event;

pub enum WeatherIcon {
    Cloudy,
    CloudyAlert,
    Dust,
    Fog,
    Hail,
    Hazy,
    Hurricane,
    Lightning,
    LightningRainy,
    Night,
    NightPartlyCloudy,
    PartlyCloudy,
    PartlyLightning,
    PartlyRainy,
    PartlySnowy,
    PartlySnowyRainy,
    Pouring,
    Rainy,
    Snowy,
    SnowyHeavy,
    SnowyRainy,
    Sunny,
    SunnyAlert,
    Tornado,
    Windy,
}

impl TryFrom<u8> for WeatherIcon {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Sunny),
            1 => Ok(Self::Sunny),
            2 => Ok(Self::PartlyCloudy),
            3 => Ok(Self::PartlyCloudy),
            4 => Ok(Self::PartlyCloudy),
            5 => Ok(Self::PartlyCloudy),
            6 => Ok(Self::PartlyRainy),
            7 => Ok(Self::PartlySnowyRainy),
            8 => Ok(Self::PartlySnowy),
            9 => Ok(Self::PartlyLightning),
            10 => Ok(Self::Cloudy),
            11 => Ok(Self::Rainy),
            12 => Ok(Self::Rainy),
            13 => Ok(Self::Pouring),
            14 => Ok(Self::Rainy),
            15 => Ok(Self::SnowyRainy),
            16 => Ok(Self::Snowy),
            17 => Ok(Self::Snowy),
            18 => Ok(Self::SnowyHeavy),
            19 => Ok(Self::LightningRainy),
            22 => Ok(Self::PartlyCloudy),
            23 => Ok(Self::Hazy),
            24 => Ok(Self::Fog),
            25 => Ok(Self::Snowy),
            26 => Ok(Self::Snowy),
            27 => Ok(Self::Hail),
            28 => Ok(Self::SnowyRainy),
            29 => Ok(Self::Night),
            30 => Ok(Self::Night),
            31 => Ok(Self::Night),
            32 => Ok(Self::NightPartlyCloudy),
            33 => Ok(Self::NightPartlyCloudy),
            34 => Ok(Self::NightPartlyCloudy),
            35 => Ok(Self::NightPartlyCloudy),
            36 => Ok(Self::PartlyRainy),
            37 => Ok(Self::PartlySnowyRainy),
            38 => Ok(Self::PartlySnowy),
            39 => Ok(Self::LightningRainy),
            40 => Ok(Self::Windy),
            41 => Ok(Self::Tornado),
            42 => Ok(Self::Tornado),
            43 => Ok(Self::Dust),
            44 => Ok(Self::Dust),
            45 => Ok(Self::Dust),
            46 => Ok(Self::LightningRainy),
            47 => Ok(Self::LightningRainy),
            48 => Ok(Self::Tornado),
            _ => Err(format!("Unknown current weather icon value: {}", value)),
        }
    }
}

async fn init_consumer() -> Result<Consumer, Box<dyn Error>> {
    let conn = Connection::connect(
        "amqps://anonymous:anonymous@dd.weather.gc.ca/%2f",
        ConnectionProperties::default(),
    )
    .await?;

    let channel = conn.create_channel().await?;
    channel
        .exchange_declare(
            "xpublic",
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                passive: true,
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    let queue = channel
        .queue_declare(
            "q_anonymous.subscribe.citypage.kiiin",
            QueueDeclareOptions {
                exclusive: true,
                durable: false,
                auto_delete: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    let routing_key = "v02.post.*.WXO-DD.citypage_weather.ON.*";
    channel
        .queue_bind(
            queue.name().as_str(),
            "xpublic",
            routing_key,
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    Ok(channel
        .basic_consume(
            queue.name().as_str(),
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?)
}

async fn get_citypage(url: Url) -> Result<String, Box<dyn Error>> {
    Ok(reqwest::get(url).await?.text().await?)
}

fn citypage_weather() -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

pub async fn monitor_weather(tx: mpsc::Sender<Event>) -> Result<(), Box<dyn Error>> {
    let mut consumer = init_consumer().await?;

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let body = std::str::from_utf8(&delivery.data)?;
            let mut iter = body.split_whitespace();

            let timestamp = NaiveDateTime::parse_from_str(
                iter.next().ok_or("missing timestamp")?,
                "%Y%m%d%H%M%S%.3f",
            )?;
            let mut url = Url::parse(iter.next().ok_or("missing url")?)?;
            url.set_path(iter.next().ok_or("missing path")?);

            println!("{:?}, {},", timestamp, url);

            delivery.ack(BasicAckOptions::default()).await?;
        }
    }

    Ok(())
}
