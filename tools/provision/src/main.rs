use std::{
    ffi::{CStr, CString},
    fs::File,
    io::Write,
    ptr,
};

use esp_idf_sys::{
    self as _, esp_err_to_name, esp_vfs_spiffs_conf_t, esp_vfs_spiffs_register, esp_vfs_unregister,
    ESP_OK,
};
use log::{error, info}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let bytebeam_config = include_bytes!("../device_config.json");

    let base_path: CString = CString::new("/spiffs").unwrap();
    let configuration_spiffs = esp_vfs_spiffs_conf_t {
        base_path: base_path.as_ptr(),
        format_if_mount_failed: true,
        max_files: 5,
        partition_label: ptr::null(),
    };

    info!("Created config");

    unsafe {
        let ret = esp_vfs_spiffs_register(&configuration_spiffs);

        if ret != ESP_OK {
            error!("FAILED :( {:?}", CStr::from_ptr(esp_err_to_name(ret)));
            esp_vfs_unregister(configuration_spiffs.base_path);
            return;
        }
    }
    info!("Registred spiffs");

    let mut file = File::create("/spiffs/device_config.json").expect("created file");
    info!("Writing device_config.json");
    file.write_all(bytebeam_config)
        .expect("wrote to file successfully");

    unsafe {
        esp_vfs_unregister(configuration_spiffs.base_path);
    }

    info!("Provisioning Done!")
}
