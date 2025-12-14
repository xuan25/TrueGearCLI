use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Manager, Peripheral};
use futures::stream::StreamExt;
use tokio::sync::Mutex;
use std::error::Error;
use std::sync::Arc;
use uuid::{Uuid, uuid};

const SERVICE_UUID_CENTER: Uuid = uuid!("6e400001-b5a3-f393-e0a9-e50e24dcca9e");
const SERVICE_UUID_CENTER_CHARACTERISTICS: Uuid = uuid!("6e400002-b5a3-f393-e0a9-e50e24dcca9e");

#[derive(Clone)]
pub struct TrueGearConnection {
    peripheral: Arc<Mutex<Option<Peripheral>>>,
    write_char: Arc<Mutex<Option<btleplug::api::Characteristic>>>,
    searching: Arc<Mutex<bool>>,
    on_connected: Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>>,
}

impl TrueGearConnection {

    pub fn new() -> Self {
        TrueGearConnection {
            peripheral: Arc::new(Mutex::new(None)),
            write_char: Arc::new(Mutex::new(None)),
            searching: Arc::new(Mutex::new(false)),
            on_connected: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_on_connected<F>(&mut self, callback: F) 
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut on_connected_guard = self.on_connected.lock().await;
        *on_connected_guard = Some(Box::new(callback));
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let _ = self.ensure_connected().await;
        Ok(())
    }

    pub async fn ensure_connected(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        {
            let connected = if let Some(peripheral) = &*self.peripheral.lock().await {
                peripheral.is_connected().await?
            } else {
                false
            };

            if connected {
                return Ok(());
            }

            let mut searching_guard = self.searching.lock().await;

            if *searching_guard {
                return Err("Searching for device".into());
            }

            *searching_guard = true;
        }

        // not connected, start auto connect
        let mut self_clone = self.clone();
        tokio::spawn(async move {
            let _ = self_clone.auto_connect().await;
            *self_clone.searching.lock().await = false;

            if let Some(callback) = &*self_clone.on_connected.lock().await {
                callback();
            }
        });
        return Err("Not connected to device".into());

    }

    pub async fn auto_connect(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
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
        let scan_filter = ScanFilter {
            services: vec![
                SERVICE_UUID_CENTER
            ]
        };

        central.start_scan(scan_filter.clone()).await?;

        // Print based on whatever the event receiver outputs. Note that the event
        // receiver blocks, so in a real program, this should be run in its own
        // thread (not task, as this library does not yet use async channels).
        let mut is_connected = false;
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
                        for _ in 0..3 {
                            match self.connect_peripheral(peripheral.clone()).await {
                                Ok(_) => {
                                    tracing::info!("Successfully connected to peripheral: {:?}", name);
                                    is_connected = true;
                                    break;
                                },
                                Err(e) => {
                                    tracing::error!("Error during connection: {}", e);
                                }
                                
                            }
                        }
                        if is_connected {
                            break;
                        }
                        
                    }
                }
                _ => {}
            }
        }

        central.stop_scan().await?;

        Ok(())

    }

    async fn connect_peripheral(&mut self, peripheral: Peripheral) -> Result<(), Box<dyn Error + Send + Sync>> {
        peripheral.connect().await?;
        if let Err(e) = peripheral.discover_services().await {
            peripheral.disconnect().await?;
            return Err(format!("Failed to discover services: {}", e).into());
        }

        let services = peripheral.services();

        let Some(target_service) = services.iter().find(|s| {
            s.uuid == SERVICE_UUID_CENTER
        }) else {
            peripheral.disconnect().await?;
            return Err(format!("Failed to find the target BLE service {:?}", SERVICE_UUID_CENTER).into());
        };

        let Some(target_characteristic) = target_service.characteristics.iter().find(|c| {
            c.uuid == SERVICE_UUID_CENTER_CHARACTERISTICS
        }) else {
            peripheral.disconnect().await?;
            return Err(format!("Failed to find the target BLE characteristic {:?}", SERVICE_UUID_CENTER_CHARACTERISTICS).into());
        };

        let write_char = target_characteristic.clone();

        *self.peripheral.lock().await = Some(peripheral);
        *self.write_char.lock().await = Some(write_char);

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(peripheral) = &*self.peripheral.lock().await {
            tracing::debug!("Disconnecting from peripheral...");
            peripheral.disconnect().await?;
        }
        Ok(())
    }

    pub async fn send_data(&mut self, data: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>> {

        self.ensure_connected().await?;

        if let (Some(peripheral), Some(write_char)) = (&*self.peripheral.lock().await, &*self.write_char.lock().await) {
            peripheral.write(write_char, data, WriteType::WithoutResponse).await?;
            Ok(())
        } else {
            Err("Not connected to BLE device".into())
        }
    }

}
