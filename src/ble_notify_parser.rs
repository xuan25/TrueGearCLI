use std::error::Error;

#[derive(Clone)]
pub struct BleNotifyParser {}

const BATTARY_FULL: f32 = 4200_f32;
const BATTARY_EMPTY: f32 = 3400_f32;
// Steepness factor
const K: f32 = 12.0;
// Midpoint adjustment
const M: f32 = 0.5;

struct DeviceStatus {
    main_model: u16,
    main_battery_mv: u16,
    left_model: u16,
    left_battery_mv: u16,
    right_model: u16,
    right_battery_mv: u16,
}

impl BleNotifyParser {
    pub fn new() -> Self {
        BleNotifyParser {}
    }

    pub fn on_message_received(&self, data: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>> {
        tracing::debug!("Received data from TrueGear: {:02X?}", data);

        fn consume_bytes<'a>(
            iter: &mut std::slice::Iter<'a, u8>,
            bytes: &[u8],
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            for byte in bytes.iter() {
                match iter.next() {
                    Some(&b) if b == *byte => {}
                    Some(_) => return Err("Unknown data received from TrueGear".into()),
                    None => return Err("Incomplete data received from TrueGear".into()),
                }
            }
            Ok(())
        }

        let mut data_iter = data.iter();

        loop {
            consume_bytes(&mut data_iter, &[0x68, 0x68])?;

            match data_iter.next() {
                Some(&num_objects) => {
                    for _ in 0..num_objects {
                        self.parse_notify_object(&mut data_iter)?;
                    }
                }
                None => {
                    tracing::error!("Incomplete data received from TrueGear: {:02X?}", data);
                    break;
                }
            }

            consume_bytes(&mut data_iter, &[0x16])?;
        }

        Ok(())
    }

    fn parse_battery_level(&self, raw_level_mv: u16) -> f32 {
        if raw_level_mv == 0 {
            return 0.0;
        }

        let ratio = (raw_level_mv as f32 - BATTARY_EMPTY) / (BATTARY_FULL - BATTARY_EMPTY);
        let clamped_ratio = ratio.clamp(0.0, 1.0);

        // Logistic curve adjustment
        1.0 / (1.0 + (-K * (clamped_ratio - M)).exp())
    }

    fn parse_notify_object(
        &self,
        iter: &mut std::slice::Iter<'_, u8>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match iter.next() {
            Some(&obj_type) => {
                tracing::debug!("Parsing object of type: {:02X?}", obj_type);
                #[allow(clippy::manual_range_patterns)]
                match obj_type {
                    0x01 | 0x02 | 0x03 | 0x04 => {
                        // Shake object
                        // Skip the rest of the shake object data
                        tracing::debug!("Skip parsing shake object data");
                        for _ in 0..(16 - 1) {
                            let _ = iter.next();
                        }
                    }
                    0x10 | 0x11 | 0x12 => {
                        // Electrical object
                        // Skip the rest of the electrical object data
                        tracing::debug!("Skip parsing electrical object data");
                        for _ in 0..(16 - 1) {
                            let _ = iter.next();
                        }
                    }
                    0x81 => {
                        // Device status object
                        tracing::debug!("Parsing device status object");
                        match self.parse_device_status_notify_object(iter) {
                            Ok(device_status) => {
                                tracing::info!(
                                    "Device Status - Main {}: {:.0}% ({} mV), Left {}: {:.0}% ({} mV), Right {}: {:.0}% ({} mV)",
                                    device_status.main_model,
                                    self.parse_battery_level(device_status.main_battery_mv) * 100.0,
                                    device_status.main_battery_mv,
                                    device_status.left_model,
                                    self.parse_battery_level(device_status.left_battery_mv) * 100.0,
                                    device_status.left_battery_mv,
                                    device_status.right_model,
                                    self.parse_battery_level(device_status.right_battery_mv)
                                        * 100.0,
                                    device_status.right_battery_mv
                                );
                            }
                            Err(e) => {
                                tracing::error!("Failed to parse device status object: {}", e);
                                return Err(e);
                            }
                        }
                    }
                    _ => {
                        tracing::warn!(
                            "Unknown object type received from TrueGear: {:02X?}",
                            obj_type
                        );
                        return Err("Unknown object type".into());
                    }
                }
            }
            None => {
                tracing::error!("Incomplete object data received from TrueGear");
                return Err("Incomplete object data".into());
            }
        }
        Ok(())
    }

    fn parse_device_status_notify_object(
        &self,
        iter: &mut std::slice::Iter<'_, u8>,
    ) -> Result<DeviceStatus, Box<dyn Error + Send + Sync>> {
        fn consume_bytes<'a>(
            iter: &mut std::slice::Iter<'a, u8>,
            bytes: &[u8],
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            for byte in bytes.iter() {
                match iter.next() {
                    Some(&b) if b == *byte => {}
                    Some(_) => return Err("Unknown device status object".into()),
                    None => return Err("Incomplete device status object".into()),
                }
            }
            Ok(())
        }

        fn read_u16<'a>(
            iter: &mut std::slice::Iter<'a, u8>,
        ) -> Result<u16, Box<dyn Error + Send + Sync>> {
            let mut bytes_needed = 2;

            let mut value: u16 = 0;
            while bytes_needed > 0 {
                match iter.next() {
                    Some(&b) => {
                        value = (value << 8) | (b as u16);
                        bytes_needed -= 1;
                    }
                    None => return Err("Incomplete device status object".into()),
                }
            }
            Ok(value)
        }

        consume_bytes(iter, &[0x02, 0x03, 0x04])?;

        let main_model = read_u16(iter)?;
        let main_battery_mv = read_u16(iter)?;

        let left_model = read_u16(iter)?;
        let left_battery_mv = read_u16(iter)?;

        let right_model = read_u16(iter)?;
        let right_battery_mv = read_u16(iter)?;

        Ok(DeviceStatus {
            main_model,
            main_battery_mv,
            left_model,
            left_battery_mv,
            right_model,
            right_battery_mv,
        })
    }
}
