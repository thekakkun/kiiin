use std::error::Error;

use askama::Template;
use chrono::DateTime;
use chrono_tz::{Canada, Tz};
use futures_util::stream::StreamExt;
use msc_citypage::{
    CityPageStream, Language, SiteData,
    models::{
        common::DayName,
        current::{CurrentConditionIcon, CurrentConditions},
        forecast::{
            AbbreviatedForecast, Forecast as CityForecast, ForecastConditionIcon, ForecastIconCode,
            HourlyForecast, LopHourly, Pop, TemperatureHourly,
        },
        measurements::{
            pressure::{PressureCondition, PressureTendency},
            temperature::{Temperature, Temperatures},
        },
    },
    sites::Ontario,
};
use quick_xml::de::from_str;
use tokio::sync::mpsc;

use crate::Event;

#[derive(Template)]
#[template(path = "weather.html")]
#[derive(Debug)]
pub(crate) struct WeatherUpdate {
    pub icon: Option<CurrentConditionIcon>,
    pub condition: Option<Condition>,
    pub today: Option<Today>,
    pub forecast: Option<Vec<Forecast>>,
}

impl From<SiteData> for WeatherUpdate {
    fn from(value: SiteData) -> Self {
        Self {
            icon: value
                .current_conditions
                .icon_code
                .clone()
                .and_then(|icon_code| icon_code.value),
            condition: (&value).try_into().ok(),
            today: (&value).try_into().ok(),
            forecast: (0..5)
                .filter_map(|i| {
                    value
                        .hourly_forecast_group
                        .hourly_forecast
                        .as_slice()
                        .get(2_usize.pow(i) - 1)
                        .map(|forecast| forecast.try_into().ok())
                })
                .collect(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Condition {
    pub temperature: f64,
    pub pressure: f64,
    pub tendency: PressureTendency,
}

impl TryFrom<&SiteData> for Condition {
    type Error = &'static str;

    fn try_from(value: &SiteData) -> Result<Self, Self::Error> {
        if let CurrentConditions {
            temperature:
                Some(Temperature {
                    value: Some(temperature),
                    ..
                }),
            pressure:
                Some(PressureCondition {
                    value: Some(pressure),
                    tendency: Some(tendency),
                    ..
                }),
            ..
        } = &value.current_conditions
        {
            Ok(Self {
                temperature: *temperature,
                pressure: *pressure,
                tendency: tendency.clone(),
            })
        } else {
            Err("Not all values found")
        }
    }
}

#[derive(Debug)]
pub(crate) struct TempPercipForecast {
    pub temperature: f64,
    pub percipitation: u8,
}

impl TryFrom<&CityForecast> for TempPercipForecast {
    type Error = &'static str;

    fn try_from(value: &CityForecast) -> Result<Self, Self::Error> {
        if let CityForecast {
            temperatures:
                Temperatures {
                    temperature:
                        Temperature {
                            value: Some(temperature),
                            ..
                        },
                    ..
                },
            abbreviated_forecast:
                AbbreviatedForecast {
                    pop:
                        Pop {
                            value: percipitation,
                            ..
                        },
                    ..
                },
            ..
        } = value
        {
            Ok(Self {
                temperature: *temperature,
                percipitation: *percipitation,
            })
        } else {
            Err("temperature or percipitation chance missing.")
        }
    }
}

#[derive(Debug)]
pub(crate) struct Today(Option<TempPercipForecast>, Option<TempPercipForecast>);

impl TryFrom<&SiteData> for Today {
    type Error = &'static str;

    fn try_from(value: &SiteData) -> Result<Self, Self::Error> {
        let forecasts: Vec<_> = value.forecast_group.forecast.iter().take(2).collect();

        match forecasts
            .iter()
            .map(|forecast| &forecast.period.text_forecast_name)
            .collect::<Vec<_>>()
            .as_slice()
        {
            [Some(DayName::Today), Some(DayName::Tonight)] => {
                if let &[today, tonight] = forecasts.as_slice() {
                    Ok(Self(today.try_into().ok(), tonight.try_into().ok()))
                } else {
                    Err("Couldn't do today and tonight")
                }
            }
            [Some(DayName::Tonight), _] => {
                if let Some(&tonight) = forecasts.first() {
                    Ok(Self(None, tonight.try_into().ok()))
                } else {
                    Err("Couldn't do tonight")
                }
            }
            _ => Err("No matching forecast."),
        }
    }
}
#[derive(Debug)]
pub(crate) struct Forecast {
    pub time: DateTime<Tz>,
    pub icon: ForecastConditionIcon,
    pub temp_percip: TempPercipForecast,
}

impl TryFrom<&HourlyForecast> for Forecast {
    type Error = &'static str;

    fn try_from(value: &HourlyForecast) -> Result<Self, Self::Error> {
        if let HourlyForecast {
            icon_code: ForecastIconCode {
                value: Some(icon), ..
            },
            temperature: TemperatureHourly {
                value: temperature, ..
            },
            lop: LopHourly {
                value: percipitation,
                ..
            },
            date_time_utc: Some(date_time_utc),
            ..
        } = value
        {
            Ok(Self {
                time: date_time_utc.with_timezone(&Canada::Eastern),
                icon: *icon,
                temp_percip: TempPercipForecast {
                    temperature: *temperature,
                    percipitation: *percipitation,
                },
            })
        } else {
            Err("Couldn't")
        }
    }
}

pub(crate) async fn monitor_weather(tx: mpsc::Sender<Event>) -> Result<(), Box<dyn Error>> {
    let mut stream = CityPageStream::new(Ontario::Toronto, Language::English).await?;

    while let Some(result) = stream.next().await {
        match result {
            Ok(url) => {
                let response = reqwest::get(url).await?;
                let xml_str = response.text().await?;

                if let Ok(site_data) = from_str::<SiteData>(&xml_str) {
                    let _ = tx.send(Event::Weather(site_data.into())).await;
                }
            }
            Err(e) => eprintln!("Stream error: {}", e),
        }
    }
    Ok(())
}
