use crate::types::RigInfo;
use log::debug;
use std::str;
use ureq::post;

use serde::Serialize;

// Wavelog API v2 radio resource: https://docs.wavelog.org/developer/api-v2/radio/
// auth is a Bearer token (no "key" field in the body), and the server assigns
// the update timestamp itself, so there's no field for that either.
#[derive(Serialize)]
struct UpdateRadioPayload {
    radio: String,
    frequency: u64,
    mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    power: Option<u64>,
}

pub fn send(address: &str, token: &str, rig_info: RigInfo) -> Result<String, String> {
    let payload = UpdateRadioPayload {
        radio: rig_info.name,
        frequency: rig_info.freq,
        mode: rig_info.mode,
        power: rig_info.power,
    };

    let json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    debug!("json={}", json);
    let auth_header = format!("Bearer {token}");
    let body = post(address)
        .header("Authorization", auth_header.as_str())
        .header("Content-Type", "application/json")
        .send(&json)
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    debug!("response={body}");
    Ok(body)
}
