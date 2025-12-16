use crate::true_gear_message::{ActionType, Effect, IntensityMode, Message, Track};
use std::{collections::HashMap, sync::OnceLock};

static SHAKE_FLAG_SHIFT_MAP: OnceLock<HashMap<u8, u8>> = OnceLock::new();

#[allow(clippy::zero_prefixed_literal)]
pub fn shake_flag_shift_map() -> &'static HashMap<u8, u8> {
    SHAKE_FLAG_SHIFT_MAP.get_or_init(|| {
        let data: [(u8, u8); 40] = [
            (000, 0x3F),
            (004, 0x3E),
            (008, 0x3D),
            (012, 0x3C),
            (016, 0x3B),
            (001, 0x3A),
            (005, 0x39),
            (009, 0x38),
            (013, 0x37),
            (017, 0x36),
            //
            (100, 0x2F),
            (104, 0x2E),
            (108, 0x2D),
            (112, 0x2C),
            (116, 0x2B),
            (101, 0x2A),
            (105, 0x29),
            (109, 0x28),
            (113, 0x27),
            (117, 0x26),
            //
            (102, 0x1F),
            (106, 0x1E),
            (110, 0x1D),
            (114, 0x1C),
            (118, 0x1B),
            (103, 0x1A),
            (107, 0x19),
            (111, 0x18),
            (115, 0x17),
            (119, 0x16),
            //
            (002, 0x0F),
            (006, 0x0E),
            (010, 0x0D),
            (014, 0x0C),
            (018, 0x0B),
            (003, 0x0A),
            (007, 0x09),
            (011, 0x08),
            (015, 0x07),
            (019, 0x06),
        ];

        let map: HashMap<u8, u8> = HashMap::from_iter(data);

        map
    })
}

static ELECTRICAL_FLAG_SHIFT_MAP: OnceLock<HashMap<u8, &'static [u8]>> = OnceLock::new();

pub fn electrical_flag_shift_map() -> &'static HashMap<u8, &'static [u8]> {
    ELECTRICAL_FLAG_SHIFT_MAP.get_or_init(|| {
        let data: [(u8, &'static [u8]); 2] = [
            (000, &[0x1F, 0x1E, 0x1D, 0x1C]),
            (100, &[0x0F, 0x0E, 0x0D, 0x0C]),
        ];

        let map: HashMap<u8, &'static [u8]> = HashMap::from_iter(data);

        map
    })
}

static ON_CONNECT_MESSAGE: std::sync::OnceLock<Vec<Message>> = std::sync::OnceLock::new();

pub fn on_connected_message() -> &'static Vec<Message> {
    ON_CONNECT_MESSAGE.get_or_init(|| {
        vec![
            Message {
                method: "play_no_registered".into(),
                body: Effect {
                    name: "Connected".into(),
                    uuid: "Connected".into(),
                    keep: false,
                    priority: 0,
                    tracks: vec![Track {
                        start_time: 0,
                        end_time: 100,
                        stop_name: "".into(),
                        start_intensity: 20,
                        end_intensity: 20,
                        intensity_mode: IntensityMode::Const,
                        action_type: ActionType::Shake,
                        once: false,
                        interval: 0,
                        index: vec![
                            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
                            100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113,
                            114, 115, 116, 117, 118, 119,
                        ],
                    }],
                },
            },
            Message {
                method: "play_no_registered".into(),
                body: Effect {
                    name: "Connected".into(),
                    uuid: "Connected".into(),
                    keep: false,
                    priority: 0,
                    tracks: vec![Track {
                        start_time: 0,
                        end_time: 0,
                        stop_name: "".into(),
                        start_intensity: 30,
                        end_intensity: 0,
                        intensity_mode: IntensityMode::Fade,
                        action_type: ActionType::Electrical,
                        once: true,
                        interval: 0,
                        index: vec![0, 100],
                    }],
                },
            },
            Message {
                method: "play_no_registered".into(),
                body: Effect {
                    name: "Connected".into(),
                    uuid: "Connected".into(),
                    keep: false,
                    priority: 0,
                    tracks: vec![Track {
                        start_time: 0,
                        end_time: 100,
                        stop_name: "".into(),
                        start_intensity: 20,
                        end_intensity: 20,
                        intensity_mode: IntensityMode::Const,
                        action_type: ActionType::Shake,
                        once: false,
                        interval: 0,
                        index: vec![
                            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
                            100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113,
                            114, 115, 116, 117, 118, 119,
                        ],
                    }],
                },
            },
            Message {
                method: "play_no_registered".into(),
                body: Effect {
                    name: "Connected".into(),
                    uuid: "Connected".into(),
                    keep: false,
                    priority: 0,
                    tracks: vec![Track {
                        start_time: 0,
                        end_time: 0,
                        stop_name: "".into(),
                        start_intensity: 30,
                        end_intensity: 0,
                        intensity_mode: IntensityMode::Fade,
                        action_type: ActionType::Electrical,
                        once: true,
                        interval: 0,
                        index: vec![0, 100],
                    }],
                },
            },
        ]
    })
}
