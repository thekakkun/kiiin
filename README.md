# kiiin

<img width="2040" height="1536" alt="image" src="https://github.com/user-attachments/assets/cdeaff42-6546-48be-8c72-81f58a247ad3" />

## What is it?

The perfect dashboard for everyone! as long as you:

- have a [jailbroken e-ink Kindle](https://kindlemodding.org/)
- have a [Music Player Daemon](https://www.musicpd.org/) setup
- find it useful to have hourly forecasts for a location in Canada

kiiin consists of two main services:

- [frame](frame/):

  Runs on the Kindle. A server that displays any image sent to the endpoint.

- [photographer](photographer/):

  Runs on some machine. Detects changes in MPD playback status or weather forecast updates. Renders the dashboard html, takes a screenshot with Firefox, and
  sends it to the Kindle.

## Attributions

- Weather data from Environment and Climate Change Canada
  - [Meteorological Service of Canada Open Data](https://eccc-msc.github.io/open-data/)
- [Weather Icons](https://github.com/erikflowers/weather-icons)
