use serde::{Deserialize};

mod body_as_base64_string {
    use base64::{engine::general_purpose, Engine as _};
    use serde::de::DeserializeOwned;
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    #[allow(dead_code)]
    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        let bytes = serde_json::to_vec(value).map_err(serde::ser::Error::custom)?;
        let b64 = general_purpose::STANDARD.encode(bytes);
        serializer.serialize_str(&b64)
    }

    #[allow(dead_code)]
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: DeserializeOwned,
    {
        let b64 = String::deserialize(deserializer)?;
        let bytes = general_purpose::STANDARD
            .decode(b64)
            .map_err(de::Error::custom)?;
        // base64 bytes -> struct (from JSON)
        serde_json::from_slice(&bytes).map_err(de::Error::custom)
    }
}

mod bool_as_string {
    use serde::{Deserialize, Deserializer, Serializer, de};

    #[allow(dead_code)]
    pub fn serialize<S>(v: &bool, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(if *v { "True" } else { "False" })
    }

    #[allow(dead_code)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct ControlCommand {
    #[serde(alias = "Method")] 
    pub method: String,
    #[serde(alias = "Body", with = "body_as_base64_string")]
    pub body: ControlCommandBody,
}

#[derive(Debug, Clone, Deserialize)]
pub enum IntensityMode {
    Const,
    Fade,
    FadeInAndOut,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ActionType {
    Shake,
    Electrical,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ControlCommandTrack {
    pub start_time: u16,
    pub end_time: u16,
    pub stop_name: String,
    pub start_intensity: u16,
    pub end_intensity: u16,
    pub intensity_mode: IntensityMode,
    pub action_type: ActionType,
    #[serde(with = "bool_as_string")]
    pub once: bool,
    pub interval: u16,
    pub index: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ControlCommandBody {
    pub name: String,
    pub uuid: String,
    #[serde(with = "bool_as_string")]
    pub keep: bool,
    pub priority: u16,
    pub tracks: Vec<ControlCommandTrack>,
}
