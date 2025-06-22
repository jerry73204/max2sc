# Data Types Specification

This document describes the serialized data types used in the max2sc converter for representing Max MSP and SuperCollider structures.

## max2sc-core

Core types and traits shared across all crates.

### Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Unsupported object type: {0}")]
    UnsupportedObject(String),
    
    #[error("Invalid parameter range: {name} = {value}")]
    InvalidParameter { name: String, value: f32 },
    
    #[error("Missing required attribute: {0}")]
    MissingAttribute(String),
}
```

### Common Types

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SphericalCoord {
    pub azimuth: f32,
    pub elevation: f32,
    pub distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat {
    Mono,
    Stereo,
    Multichannel(u32),
    Ambisonic { order: u32, dimension: u32 },
}
```

### Conversion Traits

```rust
pub trait ToSuperCollider {
    type Output;
    type Error;
    
    fn to_supercollider(&self) -> Result<Self::Output, Self::Error>;
}

pub trait FromMax<T> {
    type Error;
    
    fn from_max(max_obj: T) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
```

## max2sc-max-types

Data structures for Max MSP patch format and objects.

### Patch Structure

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MaxPatch {
    pub patcher: Patcher,
    #[serde(default)]
    pub fileversion: i32,
    pub appversion: AppVersion,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Patcher {
    pub boxes: Vec<Box>,
    pub lines: Vec<Line>,
    #[serde(default)]
    pub parameters: Parameters,
    pub rect: [f32; 4],
    #[serde(default)]
    pub openinpresentation: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppVersion {
    pub major: u32,
    pub minor: u32,
    pub revision: u32,
    pub architecture: String,
    pub modernui: u32,
}
```

### Object Types

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Box {
    pub id: String,
    #[serde(rename = "box")]
    pub content: BoxContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoxContent {
    pub maxclass: String,
    #[serde(default)]
    pub text: String,
    pub numinlets: u32,
    pub numoutlets: u32,
    pub patching_rect: [f32; 4],
    #[serde(default)]
    pub args: serde_json::Value,
    #[serde(flatten)]
    pub attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Line {
    pub patchline: PatchLine,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchLine {
    pub source: [String; 2], // [object_id, outlet_index]
    pub destination: [String; 2], // [object_id, inlet_index]
}
```

### Spatial Objects

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Spat5Object {
    #[serde(flatten)]
    pub base: BoxContent,
    pub spatial_params: SpatialParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpatialParams {
    pub sources: Option<u32>,
    pub speakers: Option<u32>,
    pub dimensions: Option<u32>,
    pub order: Option<u32>,
}
```

### OSC Configuration

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct OSCConfig {
    pub commands: Vec<OSCCommand>,
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
```

## max2sc-sc-types

Data structures for SuperCollider code generation.

### SynthDef Structure

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SynthDef {
    pub name: String,
    pub params: Vec<Parameter>,
    pub ugens: Vec<UGen>,
    pub variants: Option<Vec<Variant>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub default_value: f32,
    pub rate: Rate,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Rate {
    Audio,
    Control,
    Scalar,
    Demand,
}
```

### UGen Representations

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct UGen {
    pub name: String,
    pub rate: Rate,
    pub inputs: Vec<UGenInput>,
    pub outputs: Vec<UGenOutput>,
    pub special_index: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UGenInput {
    Constant(f32),
    Parameter(String),
    UGen { ugen_index: usize, output_index: usize },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UGenOutput {
    pub rate: Rate,
}
```

### Pattern Types

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub pattern_type: PatternType,
    pub events: Vec<Event>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PatternType {
    Pbind,
    Pseq,
    Ppar,
    Routine,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub params: HashMap<String, EventValue>,
    pub duration: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventValue {
    Float(f32),
    Symbol(String),
    Array(Vec<EventValue>),
}
```

### Bus and Routing

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct BusConfig {
    pub audio_buses: Vec<AudioBus>,
    pub control_buses: Vec<ControlBus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioBus {
    pub index: u32,
    pub num_channels: u32,
    pub name: Option<String>,
    pub private: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ControlBus {
    pub index: u32,
    pub name: Option<String>,
    pub default_value: f32,
}
```

### OSC Responders

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct OSCResponder {
    pub address: String,
    pub params: Vec<OSCParam>,
    pub action: String, // SC code as string
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OSCParam {
    pub name: String,
    pub param_type: OSCParamType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OSCParamType {
    Float,
    Int,
    String,
    Symbol,
}
```

### Project Structure

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SCProject {
    pub main_file: String,
    pub synth_defs: Vec<SynthDef>,
    pub patterns: Vec<Pattern>,
    pub bus_config: BusConfig,
    pub osc_responders: Vec<OSCResponder>,
    pub init_code: String,
    pub cleanup_code: Option<String>,
}
```

## Serialization Examples

### Max Patch Fragment
```json
{
  "patcher": {
    "fileversion": 1,
    "appversion": {
      "major": 8,
      "minor": 6,
      "revision": 5,
      "architecture": "x64",
      "modernui": 1
    },
    "boxes": [
      {
        "box": {
          "id": "obj-1",
          "maxclass": "newobj",
          "text": "spat5.pan~ @outputs 8",
          "numinlets": 1,
          "numoutlets": 8,
          "patching_rect": [100.0, 200.0, 150.0, 22.0]
        }
      }
    ]
  }
}
```

### Generated SC Structure
```yaml
synth_def:
  name: "spatPan8"
  params:
    - name: "in"
      default_value: 0.0
      rate: "Audio"
    - name: "azimuth"
      default_value: 0.0
      rate: "Control"
  ugens:
    - name: "In"
      rate: "Audio"
      inputs:
        - Parameter: "in"
    - name: "PanAz"
      rate: "Audio"
      inputs:
        - UGen: { ugen_index: 0, output_index: 0 }
        - Parameter: "azimuth"
        - Constant: 8.0
```

## Usage Guidelines

1. **Serialization**: Use `serde_json` for Max patches, `serde_yaml` for SC configs
2. **Validation**: Implement `Validate` trait for all types
3. **Conversions**: Use `From`/`Into` traits between related types
4. **Defaults**: Use `#[serde(default)]` for optional fields
5. **Versioning**: Include version info in serialized formats