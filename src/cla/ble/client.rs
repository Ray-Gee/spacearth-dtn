use crate::consts::ble::{ADV_NAME, NOTIFY_CHAR_UUID, WRITE_CHAR_UUID};
use btleplug::api::{Central, Manager as _, Peripheral as _, WriteType};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

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
