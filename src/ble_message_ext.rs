use std::error::Error;

use crate::true_gear_message;

enum IntensityModeSingleTrack {
    Const,
    Fade,
}

impl true_gear_message::Message {
    pub fn write_ble_bytes_to<'a>(&self, buffer: &'a mut Vec<u8>, electical_effect_ratio: f32) -> Result<&'a Vec<u8>, Box<dyn Error + Send + Sync>> {
        Ok(self.body.write_ble_bytes_to(buffer, electical_effect_ratio)?)
    }
}

impl true_gear_message::Effect {
    pub fn write_ble_bytes_to<'a>(&self, buffer: &'a mut Vec<u8>, electical_effect_ratio: f32) -> Result<&'a Vec<u8>, Box<dyn Error + Send + Sync>> {
        // Serialize the command body into bytes suitable for BLE transmission
        buffer.extend([
            0x68, 
            0x68,
            0x00
        ]);
        
        for track in &self.tracks {
            track.write_ble_bytes_to(buffer, self.keep, self.uuid.clone(), electical_effect_ratio)?;
        }

        buffer.push(0x16);

        Ok(buffer)
    }
}

impl true_gear_message::Track {

    fn write_ble_track_object_shake<'a>(
        buffer: &'a mut Vec<u8>, 
        intensity_mode: IntensityModeSingleTrack, 
        id: u8, 
        keep: bool, 
        time_start: u16, 
        time_end: u16, 
        intensity_start: u16, 
        intensity_end: u16, 
        index: &[u8],
    ) -> Result<(), Box<dyn Error + Send + Sync>> {

        match (intensity_mode, keep) {
            (IntensityModeSingleTrack::Const, false) => {
                buffer.push(0x01);
            }
            (IntensityModeSingleTrack::Fade, false) => {
                buffer.push(0x02);
            }
            (IntensityModeSingleTrack::Const, true) => {
                buffer.push(0x03);
            }
            (IntensityModeSingleTrack::Fade, true) => {
                buffer.push(0x04);
            }
        }

        buffer.extend([
            id,
            (time_start >> 8 & 0xFF) as u8,
            (time_start & 0xFF) as u8,
            (time_end >> 8 & 0xFF) as u8,
            (time_end & 0xFF) as u8,
            (intensity_start & 0xFF) as u8,
            (intensity_end & 0xFF) as u8,
        ]);

        let mut flag_buffer = [0u8; 8];

        for &i in index {
            if let Some(&shift) = crate::predefined::shake_flag_shift_map().get(&i) {
                let byte_index = (8 - 1 - (shift / 8)) as usize;   // big endian
                let bit_index = (shift % 8) as usize;
                flag_buffer[byte_index] |= 1 << bit_index;
            }
        }

        tracing::debug!("Shake Flags: {:08b}{:08b} {:08b}{:08b} {:08b}{:08b} {:08b}{:08b}", 
            flag_buffer[0], flag_buffer[1], flag_buffer[2], flag_buffer[3],
            flag_buffer[4], flag_buffer[5], flag_buffer[6], flag_buffer[7],
        );

        buffer.extend(&flag_buffer);

        Ok(())
    }

    fn write_ble_track_object_electrical<'a>(
        buffer: &'a mut Vec<u8>, 
        intensity_mode: IntensityModeSingleTrack, 
        once: bool,
        time_start: u16, 
        time_end: u16, 
        interval: u8, 
        intensity_start: u16, 
        intensity_end: u16, 
        index: &[u8],
        electical_effect_ratio: f32,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match (intensity_mode, once) {
            (_, true) => {
                buffer.push(0x10);
            }
            (IntensityModeSingleTrack::Const, false) => {
                buffer.push(0x11);
            }
            (IntensityModeSingleTrack::Fade, false) => {
                buffer.push(0x12);
            }
        }

        let intensity_start = ((intensity_start as f32) * electical_effect_ratio) as u16;
        let intensity_end = ((intensity_end as f32) * electical_effect_ratio) as u16;

        buffer.extend([
            0x00,
            (time_start >> 8 & 0xFF) as u8,
            (time_start & 0xFF) as u8,
            (time_end >> 8 & 0xFF) as u8,
            (time_end & 0xFF) as u8,
            interval,
            0x00,
            (intensity_start >> 8 & 0xFF) as u8,
            (intensity_start & 0xFF) as u8,
            (intensity_end >> 8 & 0xFF) as u8,
            (intensity_end & 0xFF) as u8,
        ]);

        let mut flag_buffer = [0u8; 4];

        for &i in index {
            if let Some(&shifts) = crate::predefined::electrical_flag_shift_map().get(&i) {
                for &shift in shifts.iter() {
                    let byte_index = (4 - 1 - (shift / 8)) as usize;   // big endian
                    let bit_index = (shift % 8) as usize;
                    flag_buffer[byte_index] |= 1 << bit_index;
                }
            }
        }

        tracing::debug!("Electrical Flags: {:08b}{:08b} {:08b}{:08b}", 
            flag_buffer[0], flag_buffer[1], flag_buffer[2], flag_buffer[3],
        );

        buffer.extend(&flag_buffer);

        Ok(())
    }

    pub fn write_ble_bytes_to<'a>(&self, buffer: &'a mut Vec<u8>, keep: bool, _uuid: String, electical_effect_ratio: f32) -> Result<(), Box<dyn Error + Send + Sync>> {
        let action_type = self.action_type.clone();
        let intensity_mode = self.intensity_mode.clone();
        let once = self.once;

        match action_type {
            true_gear_message::ActionType::Shake => {
                let _ = true_gear_message::Track::write_ble_track_object_shake(
                    buffer, 
                    match intensity_mode {
                        true_gear_message::IntensityMode::Const => IntensityModeSingleTrack::Const,
                        true_gear_message::IntensityMode::Fade => IntensityModeSingleTrack::Fade,
                        true_gear_message::IntensityMode::FadeInAndOut => IntensityModeSingleTrack::Fade,
                    },
                    0x00,           // TODO: uuid to id mapping
                    keep, 
                    self.start_time, 
                    match intensity_mode {
                        true_gear_message::IntensityMode::FadeInAndOut => (self.start_time + self.end_time) / 2,
                        _ => self.end_time,
                    },
                    self.start_intensity, 
                    match intensity_mode {
                        true_gear_message::IntensityMode::Const => self.start_intensity,
                        _ => self.end_intensity,
                    }, 
                    &self.index
                )?;
                buffer[2] += 1;

                match intensity_mode {
                    true_gear_message::IntensityMode::FadeInAndOut => {
                        let _ = true_gear_message::Track::write_ble_track_object_shake(
                            buffer, 
                            IntensityModeSingleTrack::Fade,
                            0x00,           // TODO: uuid to id mapping
                            keep, 
                            (self.start_time + self.end_time) / 2,
                            self.end_time,
                            self.end_intensity, 
                            self.start_intensity, 
                            &self.index)?;
                            buffer[2] += 1;
                    }
                    _ => {}
                }
            },
            true_gear_message::ActionType::Electrical => {
                let _ = true_gear_message::Track::write_ble_track_object_electrical(
                    buffer, 
                    match intensity_mode {
                        true_gear_message::IntensityMode::Const => IntensityModeSingleTrack::Const,
                        true_gear_message::IntensityMode::Fade => IntensityModeSingleTrack::Fade,
                        true_gear_message::IntensityMode::FadeInAndOut => IntensityModeSingleTrack::Fade,
                    },
                    once,
                    self.start_time, 
                    match intensity_mode {
                        true_gear_message::IntensityMode::FadeInAndOut => (self.start_time + self.end_time) / 2,
                        _ => self.end_time,
                    },
                    self.interval as u8,
                    self.start_intensity, 
                    match intensity_mode {
                        true_gear_message::IntensityMode::Const => self.start_intensity,
                        _ => self.end_intensity,
                    },
                    &self.index,
                    electical_effect_ratio
                )?;
                buffer[2] += 1;
                
                match intensity_mode {
                    true_gear_message::IntensityMode::FadeInAndOut => {
                        let _ = true_gear_message::Track::write_ble_track_object_electrical(
                            buffer, 
                            IntensityModeSingleTrack::Fade,
                            once,
                            (self.start_time + self.end_time) / 2,
                            self.end_time,
                            self.interval as u8,
                            self.end_intensity,
                            self.start_intensity,
                            &self.index,
                            electical_effect_ratio
                        )?;
                            buffer[2] += 1;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

}
