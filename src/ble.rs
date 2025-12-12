use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Manager, Peripheral};
use futures::stream::StreamExt;
use std::error::Error;
use uuid::{Uuid, uuid};

const SERVICE_UUID_CENTER: Uuid = uuid!("6e400001-b5a3-f393-e0a9-e50e24dcca9e");
const SERVICE_UUID_CENTER_CHARACTERISTICS: Uuid = uuid!("6e400002-b5a3-f393-e0a9-e50e24dcca9e");

#[derive(Clone)]
pub struct TureGearConnection {
    peripheral: Option<Peripheral>,
    write_char: Option<btleplug::api::Characteristic>,
}

impl TureGearConnection {

    pub fn new() -> Self {
        TureGearConnection {
            peripheral: None,
            write_char: None,
        }
    }

    pub async fn auto_connect(&mut self) -> Result<(), Box<dyn Error>> {
        let manager = Manager::new().await.unwrap();

        // get the first bluetooth adapter
        // connect to the adapter
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().nth(0).unwrap();

        let central_state = central.adapter_state().await.unwrap();
        tracing::debug!("CentralState: {:?}", central_state);

        // Each adapter has an event stream, we fetch via events(),
        // simplifying the type, this will return what is essentially a
        // Future<Result<Stream<Item=CentralEvent>>>.
        let mut events = central.events().await?;

        // start scanning for devices
        central.start_scan(ScanFilter::default()).await?;

        // Print based on whatever the event receiver outputs. Note that the event
        // receiver blocks, so in a real program, this should be run in its own
        // thread (not task, as this library does not yet use async channels).
        while let Some(event) = events.next().await {
            match event {
                CentralEvent::DeviceDiscovered(id) => {
                    let peripheral = central.peripheral(&id).await?;
                    let properties = peripheral.properties().await?;
                    let name = properties
                        .and_then(|p| p.local_name)
                        .map(|local_name| format!("Name: {local_name}"))
                        .unwrap_or_default();
                    tracing::debug!("DeviceDiscovered: {:?} {}", id, name);
                    if name.contains("Truegear_C") {                                                
                        tracing::debug!("Truegear_C device found: {:?}", name);
                        match self.connect_peripheral(peripheral).await {
                            Ok(_) => {
                                tracing::info!("Successfully connected to peripheral: {:?}", name);
                                break;
                            },
                            Err(e) => tracing::error!("Error during connection: {}", e),
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())

    }

    async fn connect_peripheral(&mut self, peripheral: Peripheral) -> Result<(), Box<dyn Error>> {
        peripheral.connect().await?;
        peripheral.discover_services().await?;

        let services = peripheral.services();

        let target_service = services.iter().find(|s| {
            s.uuid == SERVICE_UUID_CENTER
        }).expect(format!("Failed to find the target BLE service {:?}", SERVICE_UUID_CENTER).as_str());

        let target_characteristic = target_service.characteristics.iter().find(|c| {
            c.uuid == SERVICE_UUID_CENTER_CHARACTERISTICS
        }).expect(format!("Failed to find the target BLE characteristic {:?}", SERVICE_UUID_CENTER_CHARACTERISTICS).as_str());

        let write_char = target_characteristic.clone();

        self.peripheral = Some(peripheral);
        self.write_char = Some(write_char);

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), Box<dyn Error>> {
        if let Some(peripheral) = &self.peripheral {
            peripheral.disconnect().await?;
        }
        Ok(())
    }

    pub async fn send_data(&self, data: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let (Some(peripheral), Some(write_char)) = (&self.peripheral, &self.write_char) {
            peripheral.write(write_char, data, WriteType::WithoutResponse).await?;
            Ok(())
        } else {
            Err("Not connected to BLE device".into())
        }
    }

}
