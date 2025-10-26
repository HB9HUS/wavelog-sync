use crate::tcp::get_response;
use crate::tcp::setup_stream;
use crate::types::RigInfo;
use chrono::Utc;
use std::str;

use serde::Serialize;

#[derive(Serialize)]
struct UpdateRadioPayload {
    #[serde(rename = "key")]
    api_key: String,
    #[serde(rename = "radio")]
    radio_id: String,
    #[serde(rename = "frequency")]
    freq_hz: u64,
    #[serde(rename = "mode")]
    mode: String,
    #[serde(rename = "power", skip_serializing_if = "Option::is_none")]
    power: Option<u64>,
    #[serde(rename = "timestamp")]
    timestamp: String,
}

pub fn send(address: &str, token: &str, rig_info: RigInfo) -> Result<String, String> {
    let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let payload = UpdateRadioPayload {
        api_key: token.to_string(),
        radio_id: rig_info.name,
        freq_hz: rig_info.freq,
        mode: rig_info.mode,
        power: None, // omitted from JSON
        timestamp: timestamp,
    };

    let json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    println!("json={}", json);
    let mut stream = setup_stream(address)?;
    get_response(&mut stream, json.as_str())
}
