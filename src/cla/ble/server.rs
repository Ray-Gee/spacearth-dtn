#[cfg(target_os = "linux")]
use crate::consts::ble::{ADV_NAME, SERVICE_UUID};
#[cfg(target_os = "linux")]
use bluer::adv::Advertisement;
#[cfg(target_os = "linux")]
use tokio::time::{sleep, Duration};

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    println!("Using Bluetooth adapter: {}", adapter.name());

    // Create a simple advertisement
    let advertisement = Advertisement {
        local_name: Some(ADV_NAME.to_string()),
        service_uuids: vec![SERVICE_UUID.parse().unwrap()].into_iter().collect(),
        discoverable: Some(true),
        ..Default::default()
    };

    let _handle = adapter.advertise(advertisement).await?;
    println!("Advertising BLE Peripheral...");

    // For now, we'll just keep the advertising running
    // The GATT server implementation would need more complex setup
    // This is a simplified version to get the build working
    loop {
        sleep(Duration::from_secs(10)).await;
        println!("Server running...");
    }
}

#[cfg(not(target_os = "linux"))]
#[tokio::main]
async fn _main() -> anyhow::Result<()> {
    println!("BLE server is only supported on Linux");
    Ok(())
}
