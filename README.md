<h1 align="center">
    bytebeam-esp-rs-sdk 
</h1>

<div align="center">
  Connect ESP32 with Bytebeam IoT platform using Rust 🦀
</div>

<br />

<div align="center">
  <!-- Twitter -->
  <a href="https://twitter.com/bytebeamhq">
    <img src="https://img.shields.io/badge/twitter-%40bytebeamhq-blue?style=for-the-badge"
      alt="@bytebeamhq" />
  </a>
  <!-- Latest version -->
  <a href="https://crates.io/crates/bytebeam-esp-rs">
    <img src="https://img.shields.io/crates/v/bytebeam-esp-rs?style=for-the-badge"
      alt="Latest versoin" />
  </a>
  <!-- GitHub stars -->
  <a href="https://github.com/bytebeamio/bytebeam-esp-rs-sdk/stargazers">
    <img src="https://img.shields.io/github/stars/bytebeamio/bytebeam-esp-rs-sdk?style=for-the-badge"
      alt="GitHub stars" />
  </a>
  <!-- Docs -->
  <a href="https://docs.rs/bytebeam-esp-rs/">
    <img src="https://img.shields.io/docsrs/bytebeam-esp-rs?style=for-the-badge"
      alt="Docs" />
  </a>
  <!-- GitHub license -->
  <a href="https://github.com/bytebeamio/bytebeam-esp-rs-sdk/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/bytebeamio/bytebeam-esp-rs-sdk?style=for-the-badge"
      alt="GitHub license" />
  </a>
</div>

<div align="center">
  <sub>Built with ❤︎ by
  <a href="https://bytebeam.io/">Bytebeam</a>
</div>

<br />


<div align="center">
  <strong>Check out 
  <a href="https://bytebeam.io/docs/rust-esp-idf">docs</a> to get started
  </strong>
</div>

<br />

Bytebeam is one stop IoT backend which let's you manage OTA updates, analytics, device-mobile communication & much more :sparkles: so that you can take your project to next level without hassel.

The **bytebeam-esp-rs-sdk** allows you to connect your ESP32 board with Bytebeam using **Rust** 🦀 . You can use any ESP32 board, we used ESP32-DevkitV1 board for testing and it worked like charm! Want to try it out? see the [examples](https://github.com/bytebeamio/bytebeam-esp-rs-sdk/blob/main/examples).

> **IMPORTANT** : `bytebeam-esp-rs` requires that the certificates file ( provided by Bytebeam cloud ) exists in SPIFFS partition with name `spiffs/device_config.json`. Check out [`/tools/provision`](https://github.com/bytebeamio/bytebeam-esp-rs-sdk/tree/main/tools/provision) to know how to flash it!


<br />

## ⚡ Running examples

Rename `cfg.toml.example` to `cfg.toml` and put your Wi-Fi credentials.

You can use [cargo espflash](https://github.com/esp-rs/espflash) to build the project and flash it. Connect your ESP board using USB and run the following command:
```sh
cargo espflash --release --monitor --partition-table <PARTITION_TABLE> --example <EXAMPLE_NAME> --erase-otadata
```
> For developing in Rust on ESP, we will need to setup rust compiler and toolchains. This can easily be done by [`espup`](https://esp-rs.github.io/book/installation/installation.html#espup).

Use the same `<PARTITION_TABLE>` which you have used for provisioning certificates!

e.g. To run [actions](https://github.com/bytebeamio/bytebeam-esp-rs-sdk/blob/main/examples/actions.rs) example, with the given [partitions_example.csv](https://github.com/bytebeamio/bytebeam-esp-rs-sdk/blob/main/partitions_example.csv) 
```sh
cargo espflash --example actions --release --monitor --partition-table ./partitions_example.csv --erase-otadata
```


<br />

## ⚙️ Advance Configs

If you want to use different version of ESP IDF, or want to change the install location, you can change `[env]` in `.cargo/config.toml`.

<br />

## 🚧 Need Help?

Found some bug or need help with something? Feel free to open issues. You can open PRs as well for contributing.


<br />
