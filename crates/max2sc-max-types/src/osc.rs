//! OSC configuration types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct OSCConfig {
    pub commands: Vec<OSCCommand>,
    pub speaker_arrays: Vec<SpeakerArray>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OSCCommand {
    pub address: String,
    pub args: Vec<OSCValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OSCValue {
    Float(f32),
    Double(f64),
    Int(i32),
    String(String),
    Bool(bool),
    List(Vec<OSCValue>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpeakerArray {
    pub bus_id: u32,
    pub format: String, // "WFS", "VBAP", "HOA", etc.
    pub name: String,
    pub speakers: Vec<Speaker>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Speaker {
    pub id: u32,
    pub position: SpeakerPosition,
    pub delay: f32,
    pub gain: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpeakerPosition {
    pub azimuth: f32,   // degrees
    pub elevation: f32, // degrees
    pub distance: f32,  // meters
}

/// Parse OSC configuration from text format
pub fn parse_osc_text(content: &str) -> Result<OSCConfig, Box<dyn std::error::Error>> {
    let mut commands = Vec::new();
    let mut speaker_arrays = Vec::new();
    let mut current_buses: HashMap<u32, BusInfo> = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        // Parse OSC address and arguments
        let parts = parse_osc_line(line);
        if parts.is_empty() {
            continue;
        }

        let address = parts[0].clone();
        let args: Vec<OSCValue> = parts[1..]
            .iter()
            .map(|part| parse_osc_value(part))
            .collect();

        // Handle bus configuration
        if address.starts_with("/bus/") {
            handle_bus_command(&address, &args, &mut current_buses);
        }

        commands.push(OSCCommand { address, args });
    }

    // Convert buses to speaker arrays
    for (bus_id, bus_info) in current_buses {
        if let Some(array) = bus_info.to_speaker_array(bus_id) {
            speaker_arrays.push(array);
        }
    }

    Ok(OSCConfig {
        commands,
        speaker_arrays,
    })
}

fn parse_osc_line(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let chars = line.chars().peekable();

    for ch in chars {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    parts.push(current);
                    current = String::new();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

fn parse_osc_value(value: &str) -> OSCValue {
    // Try to parse as string (if quoted)
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        return OSCValue::String(value[1..value.len() - 1].to_string());
    }

    // Try to parse as int first (to avoid integers being parsed as floats)
    if let Ok(i) = value.parse::<i32>() {
        return OSCValue::Int(i);
    }

    // Try to parse as float
    if let Ok(f) = value.parse::<f32>() {
        return OSCValue::Float(f);
    }

    // Default to string
    OSCValue::String(value.to_string())
}

#[derive(Debug, Default)]
struct BusInfo {
    format: Option<String>,
    name: Option<String>,
    speakers_aed: Vec<f32>, // azimuth, elevation, distance triplets
    speaker_delays: HashMap<u32, f32>,
    speaker_gains: HashMap<u32, f32>,
}

impl BusInfo {
    fn to_speaker_array(&self, bus_id: u32) -> Option<SpeakerArray> {
        if self.speakers_aed.len() % 3 != 0 {
            return None;
        }

        let mut speakers = Vec::new();
        let num_speakers = self.speakers_aed.len() / 3;

        for i in 0..num_speakers {
            let speaker_id = (i + 1) as u32;
            let azimuth = self.speakers_aed[i * 3];
            let elevation = self.speakers_aed[i * 3 + 1];
            let distance = self.speakers_aed[i * 3 + 2];

            let delay = self.speaker_delays.get(&speaker_id).copied().unwrap_or(0.0);
            let gain = self.speaker_gains.get(&speaker_id).copied().unwrap_or(0.0);

            speakers.push(Speaker {
                id: speaker_id,
                position: SpeakerPosition {
                    azimuth,
                    elevation,
                    distance,
                },
                delay,
                gain,
            });
        }

        Some(SpeakerArray {
            bus_id,
            format: self.format.clone().unwrap_or_default(),
            name: self.name.clone().unwrap_or_default(),
            speakers,
        })
    }
}

fn handle_bus_command(address: &str, args: &[OSCValue], buses: &mut HashMap<u32, BusInfo>) {
    let parts: Vec<&str> = address.split('/').collect();

    if parts.len() < 3 {
        return;
    }

    // Parse bus ID
    let bus_id = if let Ok(id) = parts[2].parse::<u32>() {
        id
    } else {
        return;
    };

    let bus_info = buses.entry(bus_id).or_default();

    if parts.len() >= 4 {
        match parts[3] {
            "format" => {
                if let Some(OSCValue::String(format)) = args.first() {
                    bus_info.format = Some(format.clone());
                }
            }
            "name" => {
                if let Some(OSCValue::String(name)) = args.first() {
                    bus_info.name = Some(name.clone());
                }
            }
            "speakers" if parts.len() >= 5 && parts[4] == "aed" => {
                // Parse speaker positions in AED format
                for arg in args {
                    if let OSCValue::Float(f) = arg {
                        bus_info.speakers_aed.push(*f);
                    }
                }
            }
            "speaker" if parts.len() >= 6 => {
                // Parse individual speaker settings
                if let Ok(speaker_id) = parts[4].parse::<u32>() {
                    match parts[5] {
                        "delay" => {
                            if let Some(OSCValue::Float(delay)) = args.first() {
                                bus_info.speaker_delays.insert(speaker_id, *delay);
                            }
                        }
                        "gain" => {
                            if let Some(OSCValue::Float(gain)) = args.first() {
                                bus_info.speaker_gains.insert(speaker_id, *gain);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
