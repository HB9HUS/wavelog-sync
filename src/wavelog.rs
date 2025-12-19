use crate::types::RigInfo;
use chrono::Utc;
use log::debug;
use std::str;
use ureq::post;

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
        power: rig_info.power,
        timestamp: timestamp,
    };

    let json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    debug!("json={}", json);
    let _ = post(address)
        //.header("Content-Type", "application/json")
        .send(&json)
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    Ok(format!("ok"))
}
