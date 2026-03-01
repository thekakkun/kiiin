# kiiin

<img width="500" alt="image" src="https://github.com/user-attachments/assets/4e09d76a-a807-4efc-aa3b-de5749ea60c5" />

## What is it?

The perfect dashboard for everyone! as long as you:

- have a [jailbroken e-ink Kindle](https://kindlemodding.org/)
- have a [Music Player Daemon](https://www.musicpd.org/) setup
- find it useful to have hourly forecasts for a location in Canada

kiiin consists of two main services:

- [frame](frame/):

  This is a server that runs on the kindle. Its main functionality is to display
  any images received on-screen.

- [photographer](photographer/):

  This service can run on any machine. When it detects changes to MPD playback
  status or receives weather updates, it re-renders the dashboard image. This is
  done by taking a screenshot with headless Firefox and sending it to the
  Kindle.

## Attributions

- Weather data from Environment and Climate Change Canada
  - [Meteorological Service of Canada Open Data](https://eccc-msc.github.io/open-data/)
- [Weather Icons](https://github.com/erikflowers/weather-icons)
