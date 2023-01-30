# bytebeam-esp-rs-sdk
SDK for connecting ESP with Bytebeam IoT platform using Rust

Check out [examples](https://github.com/bytebeamio/bytebeam-esp-rs-sdk/tree/main/examples) to see how to use it!

**IMPORTANT** : `bytebeam-esp-rs` requires that the certificates file ( provided by Bytebeam cloud ) exists in SPIFFS partition with name `spiffs/device_config.json`. Check out [`/tools/provision`](https://github.com/bytebeamio/bytebeam-esp-rs-sdk/tree/main/tools/provision) to know how to flash it!

*** 

### Try out examples

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

***

### Advance Configs

If you want to use different version of ESP IDF, or want to change the install location, you can change `[env]` in `.cargo/config.toml`

