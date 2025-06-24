use crate::bpv7::EndpointId;
use crate::consts::ble::{ADV_NAME, NOTIFY_CHAR_UUID, WRITE_CHAR_UUID};
use crate::routing::algorithm::ClaPeer;
use async_trait::async_trait;
use btleplug::api::{Central, Manager as _, Peripheral as _, WriteType};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

/// BLE-specific implementation of ClaPeer for routing
pub struct BlePeer {
    pub peer_id: EndpointId,
    pub device_name: String,
}

impl BlePeer {
    pub fn new(peer_id: EndpointId, device_name: String) -> Self {
        Self {
            peer_id,
            device_name,
        }
    }
}

#[async_trait]
impl ClaPeer for BlePeer {
    fn get_peer_endpoint_id(&self) -> EndpointId {
        self.peer_id.clone()
    }

    async fn is_reachable(&self) -> bool {
        // BLE-specific connectivity check: scan for the device
        match ble_scan_for_device(&self.device_name).await {
            Ok(found) => {
                if found {
                    println!(
                        "✅ BLE peer {} ({}) is reachable",
                        self.peer_id, self.device_name
                    );
                    true
                } else {
                    println!(
                        "❌ BLE peer {} ({}) not found in scan",
                        self.peer_id, self.device_name
                    );
                    false
                }
            }
            Err(e) => {
                println!(
                    "❌ BLE peer {} ({}) scan failed: {}",
                    self.peer_id, self.device_name, e
                );
                false
            }
        }
    }

    fn get_cla_type(&self) -> &str {
        "ble"
    }

    fn get_connection_address(&self) -> String {
        self.device_name.clone()
    }
}

/// BLE-specific connectivity check
async fn ble_scan_for_device(device_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let manager = Manager::new().await?;
    let adapter = manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or("No BLE adapter found")?;

    println!("🔍 Scanning for BLE device: {}", device_name);
    adapter.start_scan(Default::default()).await?;

    // Short scan duration for connectivity check
    time::sleep(Duration::from_secs(2)).await;

    let peripherals = adapter.peripherals().await?;
    for peripheral in peripherals {
        if let Ok(Some(props)) = peripheral.properties().await {
            if let Some(name) = props.local_name {
                if name.contains(device_name) {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

/// BLE CLA client (for symmetry with TCP)
pub struct BleClaClient {
    pub device_name: String,
}

impl BleClaClient {
    pub fn new(device_name: String) -> Self {
        Self { device_name }
    }
}

#[tokio::main]
async fn _main() -> anyhow::Result<()> {
    let manager = Manager::new().await?;
    let adapter = manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .expect("No BLE adapter");

    println!("Scanning for peripherals...");
    adapter.start_scan(Default::default()).await?;
    time::sleep(Duration::from_secs(3)).await;

    let peripherals = adapter.peripherals().await?;
    let mut maybe_peripheral = None;
    for p in peripherals {
        if let Ok(Some(props)) = p.properties().await {
            if let Some(name) = props.local_name {
                if name.contains(ADV_NAME) {
                    maybe_peripheral = Some(p);
                    break;
                }
            }
        }
    }

    let peripheral = match maybe_peripheral {
        Some(p) => p,
        None => {
            println!("No target peripheral found.");
            return Ok(());
        }
    };

    peripheral.connect().await?;
    peripheral.discover_services().await?;
    println!("Connected to peripheral.");

    let chars = peripheral.characteristics();
    let write_char = chars
        .iter()
        .find(|c| c.uuid == Uuid::parse_str(WRITE_CHAR_UUID).unwrap())
        .expect("Write char not found");
    let notify_char = chars
        .iter()
        .find(|c| c.uuid == Uuid::parse_str(NOTIFY_CHAR_UUID).unwrap())
        .expect("Notify char not found");

    peripheral.subscribe(notify_char).await?;

    let bundle_data = b"HelloBundle".to_vec();
    peripheral
        .write(write_char, &bundle_data, WriteType::WithResponse)
        .await?;
    println!("Sent bundle.");

    let mut notification_stream = peripheral.notifications().await?;
    println!("Waiting for ACK...");
    if let Some(data) = notification_stream.next().await {
        println!(
            "Received notify: {:?}",
            String::from_utf8_lossy(&data.value)
        );
    }

    peripheral.disconnect().await?;
    println!("Disconnected.");
    Ok(())
}
