use crate::bpv7::EndpointId;
use crate::cla::peer::ClaPeer;
use crate::cla::ConvergenceLayer;
use crate::consts::ble::{ADV_NAME, NOTIFY_CHAR_UUID, WRITE_CHAR_UUID};
use async_trait::async_trait;
use btleplug::api::{Central, Manager as _, Peripheral as _, WriteType};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

/// BLE-specific implementation of ClaPeer for routing
#[derive(Clone)]
pub struct BlePeer {
    pub peer_id: EndpointId,
    pub device_name: String,
    pub connection_info: Option<BleConnectionInfo>,
}

/// BLE connection information including device details
#[derive(Clone, Debug)]
pub struct BleConnectionInfo {
    pub device_name: String,
    pub mac_address: String,
    pub rssi: Option<i16>,
    pub tx_power: Option<i16>,
    pub services: Vec<Uuid>,
    pub is_connectable: bool,
}

impl BleConnectionInfo {
    pub fn new(device_name: String, mac_address: String) -> Self {
        Self {
            device_name,
            mac_address,
            rssi: None,
            tx_power: None,
            services: Vec::new(),
            is_connectable: true,
        }
    }

    pub fn display_info(&self) {
        println!("📱 BLE Device Found:");
        println!("   Name: {}", self.device_name);
        println!("   MAC Address: {}", self.mac_address);
        if let Some(rssi) = self.rssi {
            println!("   RSSI: {rssi} dBm");
        }
        if let Some(tx_power) = self.tx_power {
            println!("   TX Power: {tx_power} dBm");
        }
        println!("   Connectable: {}", self.is_connectable);
        if !self.services.is_empty() {
            println!("   Services: {:?}", self.services);
        }
        println!();
    }
}

impl BlePeer {
    pub fn new(peer_id: EndpointId, device_name: String) -> Self {
        Self {
            peer_id,
            device_name,
            connection_info: None,
        }
    }

    pub fn with_connection_info(mut self, info: BleConnectionInfo) -> Self {
        self.connection_info = Some(info);
        self
    }

    pub fn get_connection_info(&self) -> Option<&BleConnectionInfo> {
        self.connection_info.as_ref()
    }
}

/// Scan for a BLE device by name and return its connection info if found
async fn ble_discover_device(device_name: &str) -> anyhow::Result<Option<BleConnectionInfo>> {
    let manager = Manager::new().await?;
    let adapter = manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No BLE adapter found"))?;

    println!("🔍 Scanning for BLE device: {device_name}");
    adapter.start_scan(Default::default()).await?;
    time::sleep(Duration::from_secs(2)).await;

    let peripherals = adapter.peripherals().await?;
    for peripheral in peripherals {
        if let Ok(Some(props)) = peripheral.properties().await {
            if let Some(name) = &props.local_name {
                if name.contains(device_name) {
                    let mut connection_info =
                        BleConnectionInfo::new(name.clone(), props.address.to_string());
                    if let Some(rssi) = props.rssi {
                        connection_info.rssi = Some(rssi);
                    }
                    if let Some(tx_power) = props.tx_power_level {
                        connection_info.tx_power = Some(tx_power);
                    }
                    connection_info.is_connectable = true;
                    return Ok(Some(connection_info));
                }
            }
        }
    }
    Ok(None)
}

/// Connect to a BLE device using its connection info
async fn ble_connect_device(info: &BleConnectionInfo) -> anyhow::Result<()> {
    let manager = Manager::new().await?;
    let adapter = manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No BLE adapter found"))?;
    let peripherals = adapter.peripherals().await?;
    for peripheral in peripherals {
        if let Ok(Some(props)) = peripheral.properties().await {
            if let Some(name) = &props.local_name {
                if name == &info.device_name && props.address.to_string() == info.mac_address {
                    peripheral.connect().await?;
                    peripheral.discover_services().await?;
                    println!(
                        "Connected to BLE device: {} ({})",
                        info.device_name, info.mac_address
                    );
                    return Ok(());
                }
            }
        }
    }
    Err(anyhow::anyhow!(
        "Device not found for connection: {}",
        info.device_name
    ))
}

#[async_trait]
impl ConvergenceLayer for BlePeer {
    fn address(&self) -> String {
        self.device_name.clone()
    }
    async fn activate(&self) -> anyhow::Result<()> {
        if let Some(connection_info) = ble_discover_device(&self.device_name).await? {
            println!("✅ BLE device found and connection info retrieved:");
            connection_info.display_info();
            // 実際の接続を行う
            ble_connect_device(&connection_info).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "BLE device not found: {}",
                self.device_name
            ))
        }
    }
}

#[async_trait]
impl ClaPeer for BlePeer {
    fn get_peer_endpoint_id(&self) -> EndpointId {
        self.peer_id.clone()
    }

    async fn is_reachable(&self) -> bool {
        ble_discover_device(&self.device_name)
            .await
            .unwrap_or(None)
            .is_some()
    }

    fn get_cla_type(&self) -> &str {
        "ble"
    }

    fn get_connection_address(&self) -> String {
        if let Some(info) = &self.connection_info {
            format!("{} ({})", info.device_name, info.mac_address)
        } else {
            self.device_name.clone()
        }
    }

    fn clone_box(&self) -> Box<dyn ClaPeer> {
        Box::new(self.clone())
    }

    async fn activate(&self) -> anyhow::Result<()> {
        <Self as ConvergenceLayer>::activate(self).await
    }
}

/// BLE CLA client (for symmetry with TCP)
pub struct BleClaClient {
    pub device_name: String,
    pub connection_info: Option<BleConnectionInfo>,
}

impl BleClaClient {
    pub fn new(device_name: String) -> Self {
        Self {
            device_name,
            connection_info: None,
        }
    }

    /// Scan for the device and store connection information
    pub async fn scan_and_store_info(&mut self) -> anyhow::Result<bool> {
        if let Some(info) = ble_discover_device(&self.device_name).await? {
            self.connection_info = Some(info.clone());
            println!("✅ Device found and connection info stored:");
            info.display_info();
            Ok(true)
        } else {
            println!("❌ Device not found: {}", self.device_name);
            Ok(false)
        }
    }

    /// Get stored connection information
    pub fn get_connection_info(&self) -> Option<&BleConnectionInfo> {
        self.connection_info.as_ref()
    }

    /// Display stored connection information
    pub fn display_stored_info(&self) {
        if let Some(info) = &self.connection_info {
            println!("📱 Stored BLE Connection Info:");
            info.display_info();
        } else {
            println!("❌ No connection information stored. Run scan_and_store_info() first.");
        }
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
