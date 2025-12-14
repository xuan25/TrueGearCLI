use crate::{ble, true_gear_message::{self, Message}};
use std::{error::Error, vec};

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
    true_gear_connection: ble::TrueGearConnection,
    electical_effect_ratio: f32,
}

impl true_gear_message::Message {
    pub fn write_bytes_to<'a>(&mut self, buffer: &'a mut Vec<u8>, electical_effect_ratio: f32) -> Result<&'a Vec<u8>, Box<dyn Error + Send + Sync>> {
        Ok(self.body.write_bytes_to(buffer, electical_effect_ratio)?)
    }
}

impl true_gear_message::Effect {
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

impl true_gear_message::Track {

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
            true_gear_message::ActionType::Shake => {
                let _ = true_gear_message::Track::write_track_object_shake(
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
                        let _ = true_gear_message::Track::write_track_object_shake(
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
                let _ = true_gear_message::Track::write_track_object_electrical(
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
                        let _ = true_gear_message::Track::write_track_object_electrical(
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

    pub async fn build(electical_effect_ratio: f32) -> Self {
        let true_gear_connection = ble::TrueGearConnection::new();
        let mut true_gear_connection_clone = true_gear_connection.clone();
        let instance = TrueGearController {
            true_gear_connection,
            electical_effect_ratio,
        };
        let controller_clone = instance.clone();

        true_gear_connection_clone.set_on_connected(
            move || {
                let mut controller_clone = controller_clone.clone();
                tokio::spawn(async move {
                    controller_clone.on_connected().await;
                });
            }
        ).await;

        instance
    }

    pub async fn on_connected(&mut self)
    {
        let mut commands = vec![
            Message {
                method: "play_no_registered".into(),
                body: true_gear_message::Effect {
                    name: "Connected".into(),
                    uuid: "Connected".into(),
                    keep: false,
                    priority: 0,
                    tracks: vec![
                        true_gear_message::Track {
                            start_time: 0,
                            end_time: 100,
                            stop_name: "".into(),
                            start_intensity: 20,
                            end_intensity: 20,
                            intensity_mode: true_gear_message::IntensityMode::Const,
                            action_type: true_gear_message::ActionType::Shake,
                            once: false,
                            interval: 0,
                            index: vec![
                                0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                                10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
                                100, 101, 102, 103, 104, 105, 106, 107, 108, 109,
                                110, 111, 112, 113, 114, 115, 116, 117, 118, 119,
                            ],
                        },
                    ],
                },
            },
            Message {
                method: "play_no_registered".into(),
                body: true_gear_message::Effect {
                    name: "Connected".into(),
                    uuid: "Connected".into(),
                    keep: false,
                    priority: 0,
                    tracks: vec![
                        true_gear_message::Track {
                            start_time: 0,
                            end_time: 0,
                            stop_name: "".into(),
                            start_intensity: 30,
                            end_intensity: 0,
                            intensity_mode: true_gear_message::IntensityMode::Fade,
                            action_type: true_gear_message::ActionType::Electrical,
                            once: true,
                            interval: 0,
                            index: vec![
                                0, 100
                            ],
                        },
                    ],
                },
            },
            Message {
                method: "play_no_registered".into(),
                body: true_gear_message::Effect {
                    name: "Connected".into(),
                    uuid: "Connected".into(),
                    keep: false,
                    priority: 0,
                    tracks: vec![
                        true_gear_message::Track {
                            start_time: 0,
                            end_time: 100,
                            stop_name: "".into(),
                            start_intensity: 20,
                            end_intensity: 20,
                            intensity_mode: true_gear_message::IntensityMode::Const,
                            action_type: true_gear_message::ActionType::Shake,
                            once: false,
                            interval: 0,
                            index: vec![
                                0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                                10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
                                100, 101, 102, 103, 104, 105, 106, 107, 108, 109,
                                110, 111, 112, 113, 114, 115, 116, 117, 118, 119,
                            ],
                        },
                    ],
                },
            },
            Message {
                method: "play_no_registered".into(),
                body: true_gear_message::Effect {
                    name: "Connected".into(),
                    uuid: "Connected".into(),
                    keep: false,
                    priority: 0,
                    tracks: vec![
                        true_gear_message::Track {
                            start_time: 0,
                            end_time: 0,
                            stop_name: "".into(),
                            start_intensity: 30,
                            end_intensity: 0,
                            intensity_mode: true_gear_message::IntensityMode::Fade,
                            action_type: true_gear_message::ActionType::Electrical,
                            once: true,
                            interval: 0,
                            index: vec![
                                0, 100
                            ],
                        },
                    ],
                },
            },
        ];
        let _ = self.send_messages(&mut commands).await;
    }

    pub fn electical_effect_ratio(&self) -> f32 {
        self.electical_effect_ratio
    }

    pub fn set_electical_effect_ratio(&mut self, ratio: f32) {
        self.electical_effect_ratio = ratio;
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.true_gear_connection.start().await
    }

    pub async fn auto_connect(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.true_gear_connection.auto_connect().await
    }

    pub async fn close(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.true_gear_connection.disconnect().await
    }

    pub async fn send_messages(&mut self, commands: &mut [true_gear_message::Message]) -> Result<(), Box<dyn Error + Send + Sync>> {

        let mut buffer: Vec<u8> = Vec::new();

        for command in &mut commands.iter_mut() {
            let mut buffer_effect: Vec<u8> = Vec::new();
            command.write_bytes_to(&mut buffer_effect, self.electical_effect_ratio)?;
            buffer.extend(buffer_effect);
        }

        tracing::debug!("Sending command bytes ({}): {:02X?}", buffer.len(), buffer);

        self.true_gear_connection.send_data(&buffer).await
    }

    pub async fn send_message(&mut self, mut command: true_gear_message::Message) -> Result<(), Box<dyn Error + Send + Sync>> {

        let mut buffer: Vec<u8> = Vec::new();
        command.write_bytes_to(&mut buffer, self.electical_effect_ratio)?;

        tracing::debug!("Sending command bytes ({}): {:02X?}", buffer.len(), buffer);

        self.true_gear_connection.send_data(&buffer).await
    }

}