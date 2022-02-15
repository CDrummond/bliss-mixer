# Bliss Mixer

Simple rust app to server a HTTP API for mixing songs. This app requires an
existing SQLite database of song analysis as created by [Bliss Analyser](https://github.com/CDrummond/bliss-analyser).

The API served is intended to be used by the [Bliss LMS DSTM plugin](https://github.com/CDrummond/lms-blissmixer).


# Building

[Rust](https://www.rust-lang.org/tools/install) is require to build.

Build with `cargo build --release`


## Start server

```
$ bliss-mixer
```
