#[cfg(target_os = "linux")]
use crate::consts::ble::{ACK, ADV_NAME, NOTIFY_CHAR_UUID, SERVICE_UUID, WRITE_CHAR_UUID};
#[cfg(target_os = "linux")]
use bluer::{
    gatt::local::{
        Application, Characteristic, CharacteristicFlags, CharacteristicNotify, Service,
    },
    Address,
};
#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "linux")]
use tokio::time::{sleep, Duration};

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    println!("Using Bluetooth adapter: {}", adapter.name());

    let app = bluer::gatt::local::ApplicationBuilder::new()
        .build()
        .await?;

    let received_data = Arc::new(Mutex::new(Vec::<u8>::new()));
    let received_data_clone = received_data.clone();

    let mut service_builder =
        bluer::gatt::local::ServiceBuilder::new(SERVICE_UUID.parse().unwrap());

    let write_char = bluer::gatt::local::CharacteristicBuilder::new(
        WRITE_CHAR_UUID.parse().unwrap(),
        CharacteristicFlags::WRITE,
    )
    .write(move |value, _| {
        println!("Received bundle: {:?}", value);
        *received_data_clone.lock().unwrap() = value;
        futures::future::ready(Ok(()))
    })
    .build();

    let notify_char = bluer::gatt::local::CharacteristicBuilder::new(
        NOTIFY_CHAR_UUID.parse().unwrap(),
        CharacteristicFlags::NOTIFY,
    )
    .notify_subscribe(|_| {
        println!("Central subscribed for ACK");
        futures::future::ready(Ok(()))
    })
    .build();

    service_builder = service_builder.characteristic(write_char);
    service_builder = service_builder.characteristic(notify_char);
    let service = service_builder.build();

    app.add_service(service).await?;

    let mut adv = adapter
        .advertise(bluer::adv::Advertisement {
            local_name: Some(ADV_NAME.to_string()),
            services: vec![SERVICE_UUID.parse().unwrap()],
            ..Default::default()
        })
        .await?;

    println!("Advertising BLE Peripheral...");

    loop {
        sleep(Duration::from_secs(10)).await;

        let data = received_data.lock().unwrap().clone();
        if !data.is_empty() {
            println!("Sending ACK for data: {:?}", data);
            // Note: ACK sending would need to be implemented differently with bluer 0.17
            // This is a simplified version that may need adjustment based on actual requirements
            received_data.lock().unwrap().clear();
        }
    }
}

#[cfg(not(target_os = "linux"))]
#[tokio::main]
async fn _main() -> anyhow::Result<()> {
    println!("BLE server is only supported on Linux");
    Ok(())
}
