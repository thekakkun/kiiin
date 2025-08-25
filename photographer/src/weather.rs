use futures_lite::stream::StreamExt;
use std::error::Error;

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
            "weather_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?)
}

pub async fn monitor_weather(tx: mpsc::Sender<Event>) -> Result<(), Box<dyn Error>> {
    let mut consumer = init_consumer().await?;

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let body = std::str::from_utf8(&delivery.data)?;
            if let Some((timestamp, rest)) = body.split_once(" ")
                && let Some((base_url, path)) = rest.split_once(" ")
            {
                println!("{}, {}, {}", timestamp, base_url, path);
            }

            delivery.ack(BasicAckOptions::default()).await?;
        }
    }

    Ok(())
}
