#[cfg(target_os = "linux")]
use crate::consts::ble::{ACK, ADV_NAME, NOTIFY_CHAR_UUID, SERVICE_UUID, WRITE_CHAR_UUID};
#[cfg(target_os = "linux")]
use bluer::{
    gatt::local::{
        Application, Characteristic, CharacteristicFlags, CharacteristicNotify, Service,
    },
    Adapter, Address, Session,
};
#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "linux")]
use tokio::time::{sleep, Duration};

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let session = Session::new().await?;
    let adapter = session.default_adapter().await?;
    println!("Using Bluetooth adapter: {}", adapter.name());

    adapter.set_powered(true).await?;

    let app = Application::new();

    let received_data = Arc::new(Mutex::new(Vec::<u8>::new()));
    let received_data_clone = received_data.clone();

    let service = Service::new_primary(SERVICE_UUID.parse().unwrap());

    let write_char = Characteristic::new(
        WRITE_CHAR_UUID.parse().unwrap(),
        CharacteristicFlags::WRITE,
        move |value| {
            println!("Received bundle: {:?}", value);
            *received_data_clone.lock().unwrap() = value;
            Ok(())
        },
    );

    let notify_char = Characteristic::new_notify(
        NOTIFY_CHAR_UUID.parse().unwrap(),
        CharacteristicFlags::NOTIFY,
        |_req: CharacteristicNotify| {
            println!("Central subscribed for ACK");
            Ok(())
        },
    );

    let service = service.with_characteristics(vec![write_char, notify_char]);
    app.insert_service(service).await?;

    adapter.start_advertising(ADV_NAME, &app).await?;

    println!("Advertising BLE Peripheral...");

    loop {
        sleep(Duration::from_secs(10)).await;

        let data = received_data.lock().unwrap().clone();
        if !data.is_empty() {
            println!("Sending ACK for data: {:?}", data);
            app.notify(NOTIFY_CHAR_UUID.parse().unwrap(), ACK.to_vec())
                .await?;
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
