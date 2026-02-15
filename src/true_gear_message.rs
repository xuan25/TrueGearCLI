use serde::{Deserialize, Serialize};
use base64::Engine;

mod bool_as_string {
    use serde::{Deserialize, Deserializer, Serializer, de};

    pub fn serialize<S>(v: &bool, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(if *v { "True" } else { "False" })
    }

    pub fn deserialize<'de, D>(d: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        // expects "true"/"false"
        let s = String::deserialize(d)?;
        match s.as_str() {
            "True" => Ok(true),
            "False" => Ok(false),
            other => Err(de::Error::custom(format!("invalid bool string: {other}"))),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    #[serde(alias = "Method")]
    pub method: String,
    #[serde(alias = "Body")]
    pub body: String,
}

impl Message {
    pub fn from_text(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    pub fn get_effect(&self) -> Option<Effect> {
        if self.method != "play_no_registered" {
            return None;
        }
        let b64 = self.body.as_str();
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
        serde_json::from_slice(&bytes).ok()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Effect {
    pub name: String,
    pub uuid: String,
    #[serde(with = "bool_as_string")]
    pub keep: bool,
    pub priority: u16,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Track {
    pub start_time: u16,
    pub end_time: u16,
    pub stop_name: String,
    pub start_intensity: u16,
    pub end_intensity: u16,
    pub intensity_mode: IntensityMode,
    pub action_type: ActionType,
    #[serde(with = "bool_as_string")]
    pub once: bool,
    pub interval: u8,
    pub index: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ActionType {
    Shake,
    Electrical,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum IntensityMode {
    Const,
    Fade,
    FadeInAndOut,
}
