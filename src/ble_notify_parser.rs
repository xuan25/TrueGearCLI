use std::error::Error;

#[derive(Clone)]
pub struct BleNotifyParser {
    
}

const BATTARY_FULL: f32 = 5000.0;

struct DeviceStatus {
    main_model: u16,
    main_battery: f32,
    left_model: u16,
    left_battery: f32,
    right_model: u16,
    right_battery: f32,
}

impl BleNotifyParser {

    pub fn new() -> Self {
        BleNotifyParser {
            
        }
    }

    pub fn on_message_received(&self, data: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>> {
        tracing::debug!("Received data from TrueGear: {:02X?}", data);
        
        fn consume_bytes<'a>(iter: &mut std::slice::Iter<'a, u8>, bytes: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>> {
            for byte in bytes.iter() {
                match iter.next() {
                    Some(&b) if b == *byte => { },
                    Some(_) => {
                        return Err("Unknown data received from TrueGear".into())
                    }
                    None => {
                        return Err("Incomplete data received from TrueGear".into())
                    }
                }
            }
            Ok(())
        }

        let mut data_iter = data.iter();

        loop {
            consume_bytes(&mut data_iter, &vec![
                0x68, 0x68
            ])?;

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

            consume_bytes(&mut data_iter, &vec![
                0x16
            ])?;
        }

        Ok(())
    }

    fn parse_notify_object(&self, iter: &mut std::slice::Iter<'_, u8>) -> Result<(), Box<dyn Error + Send + Sync>> {
        match iter.next() {
            Some(&obj_type) => {
                tracing::debug!("Parsing object of type: {:02X?}", obj_type);
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
                                tracing::info!("Device Status - Main {} ({:.2}%), Left {} ({:.2}%), Right {} ({:.2}%)",
                                    device_status.main_model, device_status.main_battery, 
                                    device_status.left_model, device_status.left_battery, 
                                    device_status.right_model, device_status.right_battery);
                            }
                            Err(e) => {
                                tracing::error!("Failed to parse device status object: {}", e);
                                return Err(e);
                            }
                        }
                    }
                    _ => {
                        tracing::warn!("Unknown object type received from TrueGear: {:02X?}", obj_type);
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

    fn parse_device_status_notify_object(&self, iter: &mut std::slice::Iter<'_, u8>) -> Result<DeviceStatus, Box<dyn Error + Send + Sync>> {

        fn consume_bytes<'a>(iter: &mut std::slice::Iter<'a, u8>, bytes: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>> {
            for byte in bytes.iter() {
                match iter.next() {
                    Some(&b) if b == *byte => { },
                    Some(_) => {
                        return Err("Unknown device status object".into())
                    }
                    None => {
                        return Err("Incomplete device status object".into())
                    }
                }
            }
            Ok(())
        }

        fn read_u16<'a>(iter: &mut std::slice::Iter<'a, u8>) -> Result<u16, Box<dyn Error + Send + Sync>> {
            let mut bytes_needed = 2;

            let mut value: u16 = 0;
            while bytes_needed > 0 {
                match iter.next() {
                    Some(&b) => {
                        value = (value << 8) | (b as u16);
                        bytes_needed -= 1;
                    }
                    None => {
                        return Err("Incomplete device status object".into())
                    }
                }
            }
            Ok(value)
        }

        consume_bytes(iter, &vec![
            0x02, 0x03, 0x04
        ])?;

        let main_model = read_u16(iter)?;
        let main_battery = read_u16(iter)? as f32 / BATTARY_FULL * 100.0;

        let left_model = read_u16(iter)?;
        let left_battery = read_u16(iter)? as f32 / BATTARY_FULL * 100.0;

        let right_model = read_u16(iter)?;
        let right_battery = read_u16(iter)? as f32 / BATTARY_FULL * 100.0;

        Ok(DeviceStatus {
            main_model,
            main_battery,
            left_model,
            left_battery,
            right_model,
            right_battery,
        })
    }

}
