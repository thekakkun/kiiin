# kiiin

## What is it?

The perfect dashboard for everyone! as long as you:

- have a [jailbroken e-ink Kindle](https://kindlemodding.org/)
- have a [Music Player Daemon](https://www.musicpd.org/) setup
- live in Canada (dependent feature under construction)

kiiin consists of two pieces:

- [frame](frame/):

  Runs on the Kindle. A server that displays any image sent to the endpoint.

- [photographer](photographer/):

  Runs on some machine. Detects changes in MPD playback status or weather (under
  construction). Renders the dashboard, takes a screenshot with Firefox, and
  sends it to the Kindle.
