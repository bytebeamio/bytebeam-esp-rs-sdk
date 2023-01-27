# Provision

Clone the repo and put the certificates file in this directory with name `device_config.json`.

```sh
git clone git@github.com:bytebeamio/bytebeam-esp-rs-sdk.git
cd bytebeam-esp-rs/tools/provision
# put device_config.json here!
```
Know how to get the config file [here](https://bytebeam.io/docs/provisioning-a-device).

## Build and Flash!

You can use [cargo espflash](https://github.com/esp-rs/espflash) to build the project and flash it.

Connect your ESP board using USB and run the following command:
```sh
cargo espflash --release --monitor --partition-table <PARTITION_TABLE>
```

> For developing in Rust on ESP, we will need to setup rust compiler and toolchains. This can easily be done by [`espup`](https://esp-rs.github.io/book/installation/installation.html#espup).

Use the same `<PARTITION_TABLE>` which you will use with your application!

### Advance Configs

If you want to use different version of ESP IDF, or want to chenge the install location, you can change `[env]` in `.cargo/config.toml`
