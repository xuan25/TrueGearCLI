use serde::{Deserialize};
use base64::prelude::*;

#[derive(Deserialize)]
pub struct ControlCommand {
    #[serde(alias = "Method")] 
    pub method: String,
    #[serde(alias = "Body")] 
    pub body_base64: String,
}

impl ControlCommand {
    pub fn body(&self) -> Result<ControlCommandBody, Box<dyn std::error::Error + Send + Sync>> {
        let result = BASE64_STANDARD.decode(&self.body_base64)?;
        let body: ControlCommandBody = serde_json::from_slice(&result)?;
        Ok(body)
    }
}


pub enum IntensityMode {
    Const,
    Fade,
    FadeInAndOut,
}

pub enum ActionType {
    Shake,
    Electrical,
}


#[derive(Deserialize)]
pub struct ControlCommandTrack {
    pub start_time: u16,
    pub end_time: u16,
    pub stop_name: String,
    pub start_intensity: u16,
    pub end_intensity: u16,
    #[serde(alias = "intensity_mode")]
    pub intensity_mode_raw: String,
    #[serde(alias = "action_type")]
    pub action_type_raw: String,
    #[serde(alias = "once")]
    pub once_raw: String,
    pub interval: u16,
    pub index: Vec<u8>,
}

impl ControlCommandTrack {
    pub fn intensity_mode(&self) -> Result<IntensityMode, Box<dyn std::error::Error + Send + Sync>> {
        match self.intensity_mode_raw.as_str() {
            "Const" => Ok(IntensityMode::Const),
            "Fade" => Ok(IntensityMode::Fade),
            "FadeInAndOut" => Ok(IntensityMode::FadeInAndOut),
            _ => Err("Unknown intensity mode".into()),
        }
    }

    pub fn action_type(&self) -> Result<ActionType, Box<dyn std::error::Error + Send + Sync>> {
        match self.action_type_raw.as_str() {
            "Shake" => Ok(ActionType::Shake),
            "Electrical" => Ok(ActionType::Electrical),
            _ => Err("Unknown action type".into()),
        }
    }

    pub fn once(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match self.once_raw.as_str() {
            "true" | "True" => Ok(true),
            "false" | "False" => Ok(false),
            _ => return Err("Unknown once value".into()),
        }
    }
}

#[derive(Deserialize)]
pub struct ControlCommandBody {
    pub name: String,
    pub uuid: String,
    #[serde(alias = "keep")] 
    pub keep_raw: String,
    pub priority: u16,
    pub tracks: Vec<ControlCommandTrack>,
}

impl ControlCommandBody {
    pub fn keep(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match self.keep_raw.as_str() {
            "true" | "True" => Ok(true),
            "false" | "False" => Ok(false),
            _ => return Err("Unknown keep value".into()),
        }
    }
}