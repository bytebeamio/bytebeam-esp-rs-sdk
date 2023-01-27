use std::{
    collections::BTreeMap,
    ffi::{CStr, CString},
    fs, ptr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use anyhow::{bail, Error};
use embedded_svc::{
    mqtt::client::{Connection, Details, Event, Message, MessageImpl, QoS},
    utils::mqtt::client::ConnState,
};
use esp_idf_svc::{
    mqtt::client::{EspMqttClient, MqttClientConfiguration},
    systime::EspSystemTime,
    tls::X509,
};
use esp_idf_sys::{
    esp_err_to_name, esp_http_client_cleanup, esp_http_client_close, esp_http_client_config_t,
    esp_http_client_fetch_headers, esp_http_client_init, esp_http_client_open,
    esp_http_client_read, esp_ota_begin, esp_ota_end, esp_ota_get_next_update_partition,
    esp_ota_handle_t, esp_ota_set_boot_partition, esp_ota_write, esp_restart,
    esp_vfs_spiffs_conf_t, esp_vfs_spiffs_register, esp_vfs_unregister, EspError, ESP_OK,
    OTA_SIZE_UNKNOWN,
};
use log::{error, info};
use serde::{Deserialize, Serialize};

pub struct ByteBeamClient {
    mqtt_client: Mutex<EspMqttClient<ConnState<MessageImpl, EspError>>>,
    action_handles: Mutex<BTreeMap<String, ActionHandler>>,
    pub device_id: String,
    pub project_id: String,
    ca_cert: &'static CStr,
    device_cert: &'static CStr,
    device_key: &'static CStr,
}

#[derive(Deserialize)]
pub struct Action {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub payload: Option<String>,
}

type ActionHandler = &'static (dyn Fn(Action, &ByteBeamClient) + Send + Sync);

impl ByteBeamClient {
    pub fn init() -> anyhow::Result<Arc<Self>> {
        let base_path: CString = CString::new("/spiffs").unwrap();
        let configuration_spiffs = esp_vfs_spiffs_conf_t {
            base_path: base_path.as_ptr(),
            format_if_mount_failed: true,
            max_files: 5,
            partition_label: ptr::null(),
        };

        unsafe {
            let ret = esp_vfs_spiffs_register(&configuration_spiffs);

            if ret != ESP_OK {
                esp_vfs_unregister(configuration_spiffs.base_path);
                bail!("FAILED :( {:?}", CStr::from_ptr(esp_err_to_name(ret)));
            }
        }

        let config = fs::read_to_string("/spiffs/device_config.json")?;

        unsafe {
            esp_vfs_unregister(configuration_spiffs.base_path);
        }

        let device_config: DeviceConfig = serde_json::from_str(&config)?;

        let ca_cert = Box::leak(
            device_config
                .authentication
                .ca_certificate
                .into_boxed_c_str(),
        );
        let device_cert = Box::leak(
            device_config
                .authentication
                .device_certificate
                .into_boxed_c_str(),
        );
        let device_key = Box::leak(
            device_config
                .authentication
                .device_private_key
                .into_boxed_c_str(),
        );

        let mqtt_config = MqttClientConfiguration {
            // client_id: todo!(),
            server_certificate: Some(X509::pem(ca_cert)),
            client_certificate: Some(X509::pem(device_cert)),
            private_key: Some(X509::pem(device_key)),
            ..Default::default()
        };

        let broker_uri = format!("mqtts://{}:{}", device_config.broker, device_config.port);
        let actions_topic = format!(
            "/tenants/{}/devices/{}/actions",
            device_config.project_id, device_config.device_id
        );

        let (mqtt_client, mut connection) = EspMqttClient::new_with_conn(broker_uri, &mqtt_config)?;

        let action_handles = BTreeMap::new();
        let bytebeam_client = ByteBeamClient {
            action_handles: Mutex::new(action_handles),
            mqtt_client: Mutex::new(mqtt_client),
            device_id: device_config.device_id,
            project_id: device_config.project_id,
            ca_cert,
            device_cert,
            device_key,
        };

        let bytebeam_client = Arc::new(bytebeam_client);

        let (tx, rx) = std::sync::mpsc::channel::<Action>();
        let cloned_client = bytebeam_client.clone();
        thread::spawn(move || {
            let bytebeam_client = cloned_client;
            info!("MQTT Listening for messages");
            while let Some(message_event) = connection.next() {
                match message_event {
                    Ok(Event::Received(data)) => {
                        if data.details() == &Details::Complete {
                            if let Ok(action) = serde_json::from_slice::<Action>(data.data()) {
                                if tx.send(action).is_err() {
                                    error!("Failed to send action")
                                };
                            };
                        }
                    }
                    Ok(Event::Connected(_)) => {
                        // subscribe to actions
                        if bytebeam_client
                            .mqtt_client
                            .lock()
                            .unwrap()
                            .subscribe(&actions_topic, QoS::AtLeastOnce)
                            .is_ok()
                        {
                            info!("subscribed to actions")
                        }
                        // register firmware update action handler
                        bytebeam_client
                            .register_action_handle("update_firmware".into(), &handle_ota);
                    }
                    _ => info!("EVENT: {message_event:?}"),
                };
            }

            error!("MQTT connection loop exit");
        });

        // thread to execute actions
        let cloned_client = bytebeam_client.clone();
        thread::spawn(move || -> anyhow::Result<()> {
            let bytebeam_client = cloned_client;
            loop {
                let action = rx.recv()?;
                if let Some(action_fn) = bytebeam_client
                    .action_handles
                    .lock()
                    .unwrap()
                    .get(&action.name)
                {
                    action_fn(action, &bytebeam_client)
                } else {
                    error!("Action handle does not exists for {}", action.name)
                }
            }
        });

        Ok(bytebeam_client)
    }

    pub fn publish_to_stream(&self, stream_name: &str, payload: &[u8]) -> anyhow::Result<u32> {
        let publish_topic = format!(
            "/tenants/{}/devices/{}/events/{}/jsonarray",
            self.project_id, self.device_id, stream_name
        );

        self.mqtt_client
            .lock()
            .unwrap()
            .publish(&publish_topic, QoS::AtLeastOnce, false, payload)
            .map_err(Error::msg)
    }

    pub fn register_action_handle(&self, action_name: String, action_function: ActionHandler) {
        info!("setting action handler for {action_name}");
        self.action_handles
            .lock()
            .unwrap()
            .insert(action_name, action_function);
    }

    pub fn publish_action_status(
        &self,
        action_id: &str,
        percentage: u32,
        status: &str,
        error_messages: Option<&[&str]>,
    ) -> anyhow::Result<u32> {
        let publish_topic = format!(
            "/tenants/{}/devices/{}/action/status",
            self.project_id, self.device_id
        );

        let errors = error_messages.unwrap_or(&[]);
        let timestamp = EspSystemTime {}.now().as_millis();

        let action_status = ActionStatus {
            id: action_id,
            errors,
            progress: percentage,
            state: status,
            timestamp,
        };

        let action_status = [action_status];

        // NOTE: convert to string if we want to log it
        // let payload = serde_json::to_string(&action_status)?;
        // println!("status payload: {payload}");

        let payload = serde_json::to_vec(&action_status)?;
        self.mqtt_client
            .lock()
            .unwrap()
            .publish(&publish_topic, QoS::AtLeastOnce, false, &payload)
            .map_err(Error::msg)
    }
}

fn handle_ota(action: Action, bytebeam_client: &ByteBeamClient) {
    if action.payload.is_none() {
        error!("Update firmware must have a payload");
        return;
    }
    let ota = serde_json::from_str(&action.payload.unwrap());

    if ota.is_err() {
        error!("Failed to deserialize payload for OTA");
        return;
    }

    let ota: Ota = ota.unwrap();

    info!("upgrading firmare version to {}", ota.version);
    let mut buf = [0; 512];

    let the_config: esp_http_client_config_t = esp_http_client_config_t {
        url: ota.url.as_ptr(),
        cert_pem: bytebeam_client.ca_cert.as_ptr(),
        client_cert_pem: bytebeam_client.device_cert.as_ptr(),
        client_key_pem: bytebeam_client.device_key.as_ptr(),
        ..Default::default()
    };

    unsafe {
        info!("Initialzing client");
        let client = esp_http_client_init(&the_config);

        info!("Opening http client");
        if esp_http_client_open(client, 0) != ESP_OK {
            error!("Failed to open connection!");
            esp_http_client_cleanup(client);
            return;
        }

        let partition = esp_ota_get_next_update_partition(ptr::null());
        let mut ota_handle: esp_ota_handle_t = 0;

        let ret = esp_ota_begin(partition, OTA_SIZE_UNKNOWN as usize, &mut ota_handle);
        if ret != ESP_OK {
            error!("Can't begin OTA due to error code {ret}");
            esp_http_client_cleanup(client);
            return;
        }
        info!("Started OTA");

        let content_length = esp_http_client_fetch_headers(client);
        let mut total_read = 0;
        let mut seq: f32 = 1.0;
        while total_read < content_length {
            let len_read = esp_http_client_read(client, buf.as_mut_ptr() as _, buf.len() as _);
            if len_read < 0 {
                error!("failed to read");
                esp_http_client_close(client);
                esp_http_client_cleanup(client);
                return;
            }
            let ret = esp_ota_write(ota_handle, buf.as_ptr() as _, len_read as usize);
            if ret != ESP_OK {
                error!("failed to write with error code {ret}");
                esp_http_client_close(client);
                esp_http_client_cleanup(client);
                return;
            }
            total_read += len_read;
            let percentage = (total_read as f32 / content_length as f32) * 100.0;
            if percentage / 10.0 >= seq {
                let state = if percentage == 100_f32 {
                    "Completed"
                } else {
                    "Progress"
                };
                info!("{percentage}% done");

                if bytebeam_client
                    .publish_action_status(&action.id, percentage as u32, state, None)
                    .is_err()
                {
                    error!("Failed to publish action status");
                    esp_http_client_close(client);
                    esp_http_client_cleanup(client);
                    return;
                };
                seq += 1.0;
            }
            buf.fill(0);
            thread::sleep(Duration::from_millis(200));
        }

        esp_http_client_close(client);
        esp_http_client_cleanup(client);
        info!("finishing up OTA");
        let ret = esp_ota_end(ota_handle);
        if ret != ESP_OK {
            error!("failed to end ota with error code {ret}");
            return;
        }
        info!("changing boot partition");
        let ret = esp_ota_set_boot_partition(partition);
        if ret != ESP_OK {
            error!("failed to write with error code {ret}");
            return;
        }

        info!("Restarting in 1 secs...");
        thread::sleep(Duration::from_secs(1));
        esp_restart();
    }
}

#[derive(Deserialize)]
struct Ota {
    url: CString,
    version: String,
    #[allow(unused)]
    status: bool,
    #[serde(rename = "content-length")]
    #[allow(unused)]
    content_length: u64,
}

#[derive(Deserialize)]
struct DeviceConfig {
    project_id: String,
    broker: String,
    port: u32,
    device_id: String,
    authentication: Auth,
}

#[derive(Serialize)]
struct ActionStatus<'a> {
    id: &'a str,
    timestamp: u128,
    errors: &'a [&'a str],
    progress: u32,
    state: &'a str,
}

#[derive(Deserialize)]
struct Auth {
    ca_certificate: CString,
    device_certificate: CString,
    device_private_key: CString,
}
