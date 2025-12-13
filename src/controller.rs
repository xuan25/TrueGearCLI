use crate::{ble, command::{ActionType, ControlCommand, ControlCommandBody, ControlCommandTrack, IntensityMode}};
use std::{error::Error};

const FRONT_EFFECT_DOT1: [u8; 20] = [
    1, 1, 4, 4, 
    1, 1, 4, 4, 
    1, 1, 4, 4, 
    1, 1, 4, 4, 
    1, 1, 4, 4
];

const FRONT_EFFECT_DOT2: [u16; 20] = [
    1 << 15, 1 << 10, 1 << 15, 1 << 10,
    1 << 14, 1 << 9,  1 << 14, 1 << 9,
    1 << 13, 1 << 8,  1 << 13, 1 << 8,
    1 << 12, 1 << 7,  1 << 12, 1 << 7,
    1 << 11, 1 << 6,  1 << 11, 1 << 6,
];

const BACK_EFFECT_DOT1: [u8; 20] = [
    2, 2, 3, 3, 
    2, 2, 3, 3, 
    2, 2, 3, 3, 
    2, 2, 3, 3, 
    2, 2, 3, 3
];

const BACK_EFFECT_DOT2: [u16; 20] = [
    1 << 15, 1 << 10, 1 << 15, 1 << 10,
    1 << 14, 1 << 9,  1 << 14, 1 << 9,
    1 << 13, 1 << 8,  1 << 13, 1 << 8,
    1 << 12, 1 << 7,  1 << 12, 1 << 7,
    1 << 11, 1 << 6,  1 << 11, 1 << 6
];

enum IntensityModeSingleTrack {
    Const,
    Fade,
}

#[derive(Clone)]
pub struct TrueGearController {
    true_gear_connection: ble::TureGearConnection,
    electical_effect_ratio: f32,
}

impl ControlCommand {
    pub fn write_bytes_to<'a>(&mut self, buffer: &'a mut Vec<u8>, electical_effect_ratio: f32) -> Result<&'a Vec<u8>, Box<dyn Error + Send + Sync>> {
        Ok(self.body.write_bytes_to(buffer, electical_effect_ratio)?)
    }
}

impl ControlCommandBody {
    pub fn write_bytes_to<'a>(&self, buffer: &'a mut Vec<u8>, electical_effect_ratio: f32) -> Result<&'a Vec<u8>, Box<dyn Error + Send + Sync>> {
        // Serialize the command body into bytes suitable for BLE transmission
        buffer.extend([
            0x68, 
            0x68,
            0x00
        ]);
        
        for track in &self.tracks {
            track.write_bytes_to(buffer, self.keep, self.uuid.clone(), electical_effect_ratio)?;
        }

        buffer.push(0x16);

        Ok(buffer)
    }
}

impl ControlCommandTrack {

    fn write_track_object_shake<'a>(
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

        let mut dot_group_front_left: u16 = 0;
        let mut dot_group_back_left: u16 = 0;
        let mut dot_group_back_right: u16 = 0;
        let mut dot_group_front_right: u16 = 0;

        for &i in index {
            // if index < 100, it's a front dot 
            if i < 100 {
                let b_usize = i as usize;
                let group = FRONT_EFFECT_DOT1[b_usize];
                let idx2hex = FRONT_EFFECT_DOT2[b_usize];
                if group == 1 {
                    dot_group_front_left |= idx2hex;
                }
                if group == 4 {
                    dot_group_front_right |= idx2hex;
                }
            // if index >= 100, it's a back dot
            } else {
                let b_usize = (i - 100) as usize;
                let groupx = BACK_EFFECT_DOT1[b_usize];
                let idx2hexx = BACK_EFFECT_DOT2[b_usize];
                if groupx == 2 {
                    dot_group_back_left |= idx2hexx;
                }
                if groupx == 3 {
                    dot_group_back_right |= idx2hexx;
                }
            }
        }

        tracing::debug!("Dot Groups - FL: {:016b}, BL: {:016b}, BR: {:016b}, FR: {:016b}", dot_group_front_left, dot_group_back_left, dot_group_back_right, dot_group_front_right);

        buffer.extend([
            ((dot_group_front_left >> 8) & 0xFF) as u8,
            (dot_group_front_left & 0xFF) as u8,
            ((dot_group_back_left >> 8) & 0xFF) as u8,
            (dot_group_back_left & 0xFF) as u8,
            ((dot_group_back_right >> 8) & 0xFF) as u8,
            (dot_group_back_right & 0xFF) as u8,
            ((dot_group_front_right >> 8) & 0xFF) as u8,
            (dot_group_front_right & 0xFF) as u8,
        ]);

        Ok(())
    }

    fn write_track_object_electrical<'a>(
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

        let mut left_index: u8 = 0;
        let mut right_index: u8 = 0;

        for &i in index {
            if i < 100 {
                left_index |= 0xF0;
            } else {
                right_index |= 0xF0;
            }
        }

        buffer.extend([
            left_index,
            0x00,
            right_index,
            0x00,
        ]);

        Ok(())
    }

    pub fn write_bytes_to<'a>(&self, buffer: &'a mut Vec<u8>, keep: bool, _uuid: String, electical_effect_ratio: f32) -> Result<(), Box<dyn Error + Send + Sync>> {
        let action_type = self.action_type.clone();
        let intensity_mode = self.intensity_mode.clone();
        let once = self.once;

        match action_type {
            ActionType::Shake => {
                let _ = ControlCommandTrack::write_track_object_shake(
                    buffer, 
                    match intensity_mode {
                        IntensityMode::Const => IntensityModeSingleTrack::Const,
                        IntensityMode::Fade => IntensityModeSingleTrack::Fade,
                        IntensityMode::FadeInAndOut => IntensityModeSingleTrack::Fade,
                    },
                    0x00,           // TODO: uuid to id mapping
                    keep, 
                    self.start_time, 
                    match intensity_mode {
                        IntensityMode::FadeInAndOut => (self.start_time + self.end_time) / 2,
                        _ => self.end_time,
                    },
                    self.start_intensity, 
                    match intensity_mode {
                        IntensityMode::Const => self.start_intensity,
                        _ => self.end_intensity,
                    }, 
                    &self.index
                )?;
                buffer[2] += 1;

                match intensity_mode {
                    IntensityMode::FadeInAndOut => {
                        let _ = ControlCommandTrack::write_track_object_shake(
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
            ActionType::Electrical => {
                let _ = ControlCommandTrack::write_track_object_electrical(
                    buffer, 
                    match intensity_mode {
                        IntensityMode::Const => IntensityModeSingleTrack::Const,
                        IntensityMode::Fade => IntensityModeSingleTrack::Fade,
                        IntensityMode::FadeInAndOut => IntensityModeSingleTrack::Fade,
                    },
                    once,
                    self.start_time, 
                    match intensity_mode {
                        IntensityMode::FadeInAndOut => (self.start_time + self.end_time) / 2,
                        _ => self.end_time,
                    },
                    self.interval as u8,
                    self.start_intensity, 
                    match intensity_mode {
                        IntensityMode::Const => self.start_intensity,
                        _ => self.end_intensity,
                    },
                    &self.index,
                    electical_effect_ratio
                )?;
                buffer[2] += 1;
                
                match intensity_mode {
                    IntensityMode::FadeInAndOut => {
                        let _ = ControlCommandTrack::write_track_object_electrical(
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

impl TrueGearController {

    pub fn new() -> Self {
        TrueGearController {
            true_gear_connection: ble::TureGearConnection::new(),
            electical_effect_ratio: 1.0,
        }
    }

    pub fn electical_effect_ratio(&self) -> f32 {
        self.electical_effect_ratio
    }

    pub fn set_electical_effect_ratio(&mut self, ratio: f32) {
        self.electical_effect_ratio = ratio;
    }

    pub async fn auto_connect(&mut self) -> Result<(), Box<dyn Error>> {
        self.true_gear_connection.auto_connect().await
    }

    pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        self.true_gear_connection.disconnect().await
    }

    pub async fn send_command(&self, mut command: ControlCommand) -> Result<(), Box<dyn Error + Send + Sync>> {
        
        let mut buffer: Vec<u8> = Vec::new();
        command.write_bytes_to(&mut buffer, self.electical_effect_ratio)?;

        tracing::debug!("Sending command bytes: {:02X?}", buffer);

        self.true_gear_connection.send_data(&buffer).await
    }

}