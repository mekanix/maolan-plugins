# Maolan Plugins

[![crates.io](https://img.shields.io/crates/v/maolan-plugins.svg)](https://crates.io/crates/maolan-plugins)

A collection of audio plugins written in Rust for the Maolan ecosystem. All plugins implement the
[CLAP](https://cleveraudio.org/) plugin API and include an Iced-based GUI using the TokyoNight
theme.

## Build

```bash
cargo build --release
```

## Platform Support

Linux, FreeBSD, macOS, and Windows are supported.

---

## Only CLAP support

As only CLAP supports changing number of channels while plugin instance is loaded, the decision is
to support only that plugin format. With other formats one has to implement mono and stereo plugin
and those can not be swapped easily. On top of that, once there's an option for mono, stereo, 2.1,
5.1 and 7.1 the number of plugins becomes huge. Just imagine having 10 plugins with all channel
variations: that makes 50 plugins to choose from in a plugin browser. That being said, if LV2 gets
support for changing channel number, it will be implemented, but only then.

[List of plugins](https://doc.maolan.rs/plugins.html)
