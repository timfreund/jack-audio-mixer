# JACK Audio Mixer

This project mixes audio from several input channels.  Channel levels
are configurable via
[OSC](http://opensoundcontrol.org/introduction-osc).

Project status:  functional, but barely.

## Things to fix

- Channel count is hardcoded
  - You get 8 inputs
  - 2 outputs
- Channels have a pan value that is unused
- The rust is littered with unwraps and needs functions extracted from
  monolithic blocks of code.  This is my first rust program beyond "hello world"
  level work, so suggestions are welcomed.
- Messages sent between the main thread and jack callback are sent as hardcoded strings
- The OSC port is not configurable
- OSC clients should receive updates of level/pan/mute
- The OSC processing does not handle mute/pan input
