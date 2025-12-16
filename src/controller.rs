use crate::{ble, ble_notify_parser, predefined, true_gear_message};
use std::error::Error;

#[derive(Clone)]
pub struct TrueGearBLEController {
    true_gear_connection: ble::TrueGearBLEConnection,
    electical_effect_ratio: f32,
    #[allow(unused)]
    ble_notify_parser: ble_notify_parser::BleNotifyParser,
}

impl TrueGearBLEController {
    pub async fn build(electical_effect_ratio: f32) -> Self {
        let true_gear_connection = ble::TrueGearBLEConnection::new();
        let mut true_gear_connection_clone = true_gear_connection.clone();
        let ble_notify_parser = ble_notify_parser::BleNotifyParser::new();
        let instance = TrueGearBLEController {
            true_gear_connection,
            electical_effect_ratio,
            ble_notify_parser: ble_notify_parser.clone(),
        };
        let controller_clone = instance.clone();

        true_gear_connection_clone
            .set_on_connected(move || {
                let mut controller_clone = controller_clone.clone();
                tokio::spawn(async move {
                    controller_clone.on_connected().await;
                });
            })
            .await;

        true_gear_connection_clone
            .set_on_message_received(move |data: &[u8]| {
                let _ = ble_notify_parser.on_message_received(data);
            })
            .await;

        instance
    }

    pub async fn on_connected(&mut self) {
        let _ = self
            .send_ble_messages(predefined::on_connected_message())
            .await;
    }

    #[allow(dead_code)]
    pub fn electical_effect_ratio(&self) -> f32 {
        self.electical_effect_ratio
    }

    pub fn set_electical_effect_ratio(&mut self, ratio: f32) {
        self.electical_effect_ratio = ratio;
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.true_gear_connection.start().await
    }

    #[allow(dead_code)]
    pub async fn auto_connect(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.true_gear_connection.auto_connect().await
    }

    pub async fn close(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.true_gear_connection.disconnect().await
    }

    pub async fn send_ble_messages(
        &mut self,
        messages: &[true_gear_message::Message],
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut buffer: Vec<u8> = Vec::new();

        for message in &mut messages.iter() {
            let mut buffer_effect: Vec<u8> = Vec::new();
            message.write_ble_bytes_to(&mut buffer_effect, self.electical_effect_ratio)?;
            buffer.extend(buffer_effect);
        }

        tracing::debug!("Sending message bytes ({}): {:02X?}", buffer.len(), buffer);

        self.true_gear_connection.send_data(&buffer).await
    }

    pub async fn send_ble_message(
        &mut self,
        message: true_gear_message::Message,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut buffer: Vec<u8> = Vec::new();
        message.write_ble_bytes_to(&mut buffer, self.electical_effect_ratio)?;

        tracing::debug!("Sending message bytes ({}): {:02X?}", buffer.len(), buffer);

        self.true_gear_connection.send_data(&buffer).await
    }
}
