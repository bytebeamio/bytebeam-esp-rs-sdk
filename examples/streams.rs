use anyhow::bail;
use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_idf_hal::interrupt;
use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sntp::{self, SyncStatus};
use esp_idf_svc::systime::EspSystemTime;
use esp_idf_svc::wifi::{EspWifi, WifiWait};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use std::cell::RefCell;
use std::sync::Mutex;
use std::time::Duration;

use bytebeam_esp_rs::ByteBeamClient;
use esp_idf_hal::gpio::{Gpio2, Output, PinDriver};
use esp_idf_hal::peripherals::Peripherals;

static ONBOARD_LED: Mutex<RefCell<Option<PinDriver<Gpio2, Output>>>> =
    Mutex::new(RefCell::new(None));

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let _wifi = connect_wifi(peripherals.modem, sysloop.clone(), nvs)?;

    let sntp = sntp::EspSntp::new_default().unwrap();
    while sntp.get_sync_status() != SyncStatus::Completed {}
    println!("SNTP Initialized");

    let pin2 = peripherals.pins.gpio2;
    let pin2_driver = PinDriver::output(pin2)?;
    interrupt::free(|| ONBOARD_LED.lock().unwrap().replace(Some(pin2_driver)));

    // Bytebeam!
    let bytebeam_client = ByteBeamClient::init()?;

    let timestamp = EspSystemTime {}.now().as_millis().to_string();
    let sequence = 1;
    let message = MyStream {
        id: bytebeam_client.device_id.clone(),
        sequence,
        timestamp,
        status: "ON".into(),
    };

    let message = [message];

    let payload = serde_json::to_vec(&message).unwrap();

    bytebeam_client
        .publish_to_stream("example", &payload)
        .expect("published successfully");

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn connect_wifi(
    modem: Modem,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> anyhow::Result<EspWifi<'static>> {
    let wifi_configs = CONFIG;

    let mut wifi_driver = EspWifi::new(modem, sysloop.clone(), Some(nvs))?;

    let ap_infos = wifi_driver.scan()?;

    let ours = ap_infos
        .into_iter()
        .find(|a| a.ssid == wifi_configs.wifi_ssid);

    let channel = if let Some(ours) = ours {
        Some(ours.channel)
    } else {
        None
    };

    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: wifi_configs.wifi_ssid.into(),
        password: wifi_configs.wifi_psk.into(),
        channel,
        ..Default::default()
    }))?;

    wifi_driver.start()?;

    if !WifiWait::new(&sysloop)?.wait_with_timeout(Duration::from_secs(20), || {
        wifi_driver.is_started().unwrap()
    }) {
        bail!("Wifi did not start");
    }

    wifi_driver.connect()?;

    while !wifi_driver.is_connected()? {
        std::thread::sleep(Duration::from_millis(200));
    }

    Ok(wifi_driver)
}

use serde::Serialize;

#[derive(Serialize)]
struct MyStream {
    // expected by default
    id: String,
    sequence: u32,
    timestamp: String,
    // your custom fields!
    status: String,
}
