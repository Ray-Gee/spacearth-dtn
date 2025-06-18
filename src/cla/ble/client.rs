use btleplug::api::{Central, Manager as _, Peripheral as _, WriteType, Characteristic};
use btleplug::platform::Manager;
use uuid::Uuid;
use std::time::Duration;
use tokio::time;
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = Manager::new().await?;
    let adapter = manager.adapters().await?.into_iter().next().expect("No BLE adapter");

    println!("Scanning for peripherals...");
    adapter.start_scan().await?;
    time::sleep(Duration::from_secs(3)).await;

    let peripherals = adapter.peripherals().await?;
    let maybe_peripheral = peripherals.into_iter().find(|p| {
        p.properties()
            .ok()
            .and_then(|props| props.local_name)
            .filter(|name| name.contains("spacearth-dtn-ble"))
            .is_some()
    });

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
    let write_char = chars.iter().find(|c| c.uuid == Uuid::parse_str("12345678-1234-5678-1234-56789abcdef1").unwrap()).expect("Write char not found");
    let notify_char = chars.iter().find(|c| c.uuid == Uuid::parse_str("12345678-1234-5678-1234-56789abcdef2").unwrap()).expect("Notify char not found");

    peripheral.subscribe(notify_char).await?;

    let bundle_data = b"HelloBundle".to_vec();
    peripheral.write(write_char, &bundle_data, WriteType::WithResponse).await?;
    println!("Sent bundle.");

    let mut notification_stream = peripheral.notifications().await?;
    println!("Waiting for ACK...");
    while let Some(data) = notification_stream.next().await {
        println!("Received notify: {:?}", String::from_utf8_lossy(&data.value));
        break;
    }

    peripheral.disconnect().await?;
    println!("Disconnected.");
    Ok(())
}
