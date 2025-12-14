use crate::true_gear_message::{ActionType, Effect, IntensityMode, Message, Track};

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
                    tracks: vec![
                        Track {
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
                body: Effect {
                    name: "Connected".into(),
                    uuid: "Connected".into(),
                    keep: false,
                    priority: 0,
                    tracks: vec![
                        Track {
                            start_time: 0,
                            end_time: 0,
                            stop_name: "".into(),
                            start_intensity: 30,
                            end_intensity: 0,
                            intensity_mode: IntensityMode::Fade,
                            action_type: ActionType::Electrical,
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
                body: Effect {
                    name: "Connected".into(),
                    uuid: "Connected".into(),
                    keep: false,
                    priority: 0,
                    tracks: vec![
                        Track {
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
                body: Effect {
                    name: "Connected".into(),
                    uuid: "Connected".into(),
                    keep: false,
                    priority: 0,
                    tracks: vec![
                        Track {
                            start_time: 0,
                            end_time: 0,
                            stop_name: "".into(),
                            start_intensity: 30,
                            end_intensity: 0,
                            intensity_mode: IntensityMode::Fade,
                            action_type: ActionType::Electrical,
                            once: true,
                            interval: 0,
                            index: vec![
                                0, 100
                            ],
                        },
                    ],
                },
            },
        ]
    })
}
