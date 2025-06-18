use bluer::{Adapter, Address, gatt::local::{Application, Characteristic, CharacteristicNotify, Service}};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> bluer::Result<()> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    println!("Using Bluetooth adapter: {}", adapter.name());

    adapter.set_powered(true).await?;

    let app = Application::new();

    let received_data = Arc::new(Mutex::new(Vec::<u8>::new()));
    let received_data_clone = received_data.clone();

    let service = Service::new_primary("12345678-1234-5678-1234-56789abcdef0".parse().unwrap());

    let write_char = Characteristic::new(
        "12345678-1234-5678-1234-56789abcdef1".parse().unwrap(),
        bluer::gatt::local::CharacteristicFlags::WRITE,
        move |value| {
            println!("Received bundle: {:?}", value);
            *received_data_clone.lock().unwrap() = value;
            Ok(())
        },
    );

    let notify_char = Characteristic::new_notify(
        "12345678-1234-5678-1234-56789abcdef2".parse().unwrap(),
        bluer::gatt::local::CharacteristicFlags::NOTIFY,
        |_req: CharacteristicNotify| {
            println!("Central subscribed for ACK");
            Ok(())
        },
    );

    let service = service.with_characteristics(vec![write_char, notify_char]);
    app.insert_service(service).await?;

    adapter.start_advertising("spacearth-dtn-ble", &app).await?;

    println!("Advertising BLE Peripheral...");

    loop {
        sleep(Duration::from_secs(10)).await;

        let data = received_data.lock().unwrap().clone();
        if !data.is_empty() {
            println!("Sending ACK for data: {:?}", data);
            app.notify("12345678-1234-5678-1234-56789abcdef2".parse().unwrap(), b"ACK\n".to_vec()).await?;
            received_data.lock().unwrap().clear();
        }
    }
}
