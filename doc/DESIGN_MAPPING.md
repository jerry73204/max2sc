# Max MSP to SuperCollider Component Mapping

## Overview

This document details the mapping strategy from Max MSP 8 objects to SuperCollider equivalents, with special focus on spatial audio processing. The Ambisonic Toolkit (ATK) for SuperCollider is used extensively for HOA processing.

## Analysis Summary

Based on analysis of 237 Max patches in the project:
- **419** unique object types identified
- **211** instances of `spat5.osc.route` (primary routing object)
- **140+** SPAT5 spatial audio objects
- **82-channel** multichannel processing capability
- Extensive OSC namespace for control

## Core Object Mappings

### Audio I/O

| Max Object | SuperCollider Equivalent                  | Notes               |
|------------|-------------------------------------------|---------------------|
| `dac~`     | `Out.ar`                                  | Direct mapping      |
| `adc~`     | `In.ar`, `SoundIn.ar`                     | Input selection     |
| `ezadc~`   | `SoundIn.ar`                              | Simplified input    |
| `ezdac~`   | `Out.ar`                                  | Simplified output   |
| `adstatus` | `ServerOptions`, `Server.default.options` | Audio configuration |
| `mc.dac~`  | `Out.ar` with array expansion             | Multichannel output |
| `mc.adc~`  | `In.ar` with array expansion              | Multichannel input  |

### Multichannel Processing

| Max Object       | SuperCollider Equivalent               | Notes                       |
|------------------|----------------------------------------|-----------------------------|
| `mc.unpack~`     | Array indexing `sig[0..n]`             | Channel extraction          |
| `mc.pack~`       | Array construction `[sig1, sig2, ...]` | Channel combination         |
| `mc.live.gain~`  | `sig * gain.lag` with array            | Multichannel gain           |
| `mc.*` (general) | Array operations                       | Use `.collect`, `.do`, etc. |

### Basic Signal Processing

| Max Object | SuperCollider Equivalent        | Notes            |
|------------|---------------------------------|------------------|
| `*~`       | `*`                             | Multiplication   |
| `+~`       | `+`                             | Addition         |
| `cycle~`   | `SinOsc.ar`                     | Sine oscillator  |
| `noise~`   | `WhiteNoise.ar`, `PinkNoise.ar` | Noise generators |
| `sig~`     | `DC.ar`                         | Constant signal  |
| `line~`    | `Line.ar`, `XLine.ar`           | Linear ramp      |
| `number~`  | `K2A.ar`                        | Control to audio |

### Spatial Audio - Native Max

| Max Object | SuperCollider Equivalent | Notes |
|------------|-------------------------|-------|
| `pan~` | `Pan2.ar` | Stereo panning |
| `pan2~` | `Pan2.ar` | Stereo panning |
| `pan4~` | `Pan4.ar` | Quad panning |
| `pan8~` | `PanAz.ar(8, ...)` | 8-channel panning |

### Spatial Audio - SPAT5 Core

| Max Object         | SuperCollider Implementation | Notes                      |
|--------------------|------------------------------|----------------------------|
| `spat5.panoramix~` | Custom `SpatPanoramix` class | Main spatialization engine |
| `spat5.pan~`       | `VBAP.ar` or custom panning  | Flexible panning           |
| `spat5.spat~`      | Custom spatializer           | Room acoustics             |
| `spat5.viewer`     | GUI integration needed       | Visual feedback            |

### SPAT5 HOA (Higher Order Ambisonics)

| Max Object             | ATK Equivalent                       | Notes                |
|------------------------|--------------------------------------|----------------------|
| `spat5.hoa.encoder~`   | `FoaEncode.ar`, `HoaEncodeMatrix.ar` | Ambisonic encoding   |
| `spat5.hoa.decoder~`   | `FoaDecode.ar`, `HoaDecodeMatrix.ar` | Ambisonic decoding   |
| `spat5.hoa.rotate~`    | `FoaRotate.ar`, `HoaRotate.ar`       | Rotation transform   |
| `spat5.hoa.mirror~`    | `FoaMirror.ar`                       | Reflection transform |
| `spat5.hoa.focus~`     | `FoaFocus.ar`, `HoaFocus.ar`         | Beamforming          |
| `spat5.hoa.blur~`      | Custom implementation                | Spatial blur         |
| `spat5.hoa.converter~` | Format conversion matrices           | FOA/HOA conversion   |
| `spat5.hoa.binaural~`  | `FoaDecode.ar(method: 'binaural')`   | Binaural decoding    |

### SPAT5 VBAP

| Max Object | SuperCollider Equivalent | Notes |
|------------|-------------------------|-------|
| `spat5.vbap` | `VBAP.ar` | Vector-based panning |
| `spat5.vbip` | `VBIP.ar` (if available) | Intensity panning |
| `spat5.vbap3d` | `VBAP.ar` with 3D setup | 3D panning |

### SPAT5 WFS (Wave Field Synthesis)

| Max Object | SuperCollider Implementation | Notes |
|------------|----------------------------|-------|
| WFS configurations | Custom `WFSArray` class | Array management |
| Speaker delays | `DelayN.ar` arrays | Per-speaker delays |
| Pre-filtering | `BHiShelf.ar`, `BLowShelf.ar` | Compensation filters |

### SPAT5 Effects

| Max Object          | SuperCollider Equivalent          | Notes          |
|---------------------|-----------------------------------|----------------|
| `spat5.reverb~`     | `JPverb.ar`, `GVerb.ar`           | Reverberation  |
| `spat5.early~`      | Custom early reflection network   | Tapped delays  |
| `spat5.cascade~`    | Serial `BPF.ar` or `SVF.ar`       | Filter cascade |
| `spat5.compressor~` | `Compander.ar`                    | Dynamics       |
| `spat5.delay~`      | `DelayN.ar`, `DelayL.ar`          | Delay lines    |
| `spat5.equalizer~`  | `BPeakEQ.ar`, `BHiShelf.ar`, etc. | EQ bands       |

### SPAT5 Utilities

| Max Object          | SuperCollider Equivalent  | Notes                |
|---------------------|---------------------------|----------------------|
| `spat5.osc.route`   | `OSCFunc`, `OSCdef`       | OSC routing          |
| `spat5.osc.prepend` | String concatenation      | OSC address building |
| `spat5.meter~`      | `Amplitude.ar`, `Peak.ar` | Level metering       |
| `spat5.rms~`        | `RunningSum.ar`           | RMS measurement      |
| `spat5.scope~`      | `Scope`                   | Signal visualization |

### Routing and Control

| Max Object  | SuperCollider Equivalent       | Notes                 |
|-------------|--------------------------------|-----------------------|
| `gate~`     | `Select.ar`                    | Signal switching      |
| `selector~` | `Select.ar`                    | Multi-input selection |
| `matrix~`   | Custom routing matrix          | Cross-point routing   |
| `route`     | Pattern matching, conditionals | Message routing       |
| `prepend`   | String/symbol operations       | Message formatting    |

## OSC Namespace Mapping

The SPAT5/Panoramix OSC namespace should be preserved for compatibility:

### Master Control
- `/master/gain` → Master bus gain control
- `/master/mute` → Master mute
- `/master/numio` → I/O configuration

### Bus Control
- `/bus/[n]/speaker/[m]/delay` → Speaker delay
- `/bus/[n]/speaker/[m]/gain` → Speaker gain
- `/bus/[n]/format` → Bus format (WFS, VBAP, etc.)

### Track Control
- `/track/[n]/azim` → Azimuth position
- `/track/[n]/elev` → Elevation
- `/track/[n]/dist` → Distance
- `/track/[n]/doppler` → Doppler effect
- `/track/[n]/air` → Air absorption

### Reverb Control
- `/reverb/[n]/tr0` → Reverb time at DC
- `/reverb/[n]/trl` → Low frequency RT
- `/reverb/[n]/trm` → Mid frequency RT
- `/reverb/[n]/trh` → High frequency RT

## Implementation Strategy

### 1. Core Classes

```supercollider
// Base spatial processor
SpatProcessor {
    var <numInputs, <numOutputs;
    var <speakerArray;
    // Common spatial processing methods
}

// WFS implementation
WFSArray : SpatProcessor {
    var <delayTimes, <gains;
    var <prefilters;
    // Wave field synthesis methods
}

// HOA processor using ATK
HOAProcessor : SpatProcessor {
    var <order, <dimension;
    var <encoder, <decoder;
    // ATK integration
}
```

### 2. Multichannel Support

```supercollider
// Multichannel routing
MultichannelRouter {
    var <matrix;
    *ar { |inputs, routing|
        // Flexible routing implementation
    }
}
```

### 3. OSC Integration

```supercollider
// OSC namespace handler
SpatOSCRouter {
    var <responders;
    
    *new { |port = 57120|
        // Set up OSC responders matching Max namespace
    }
}
```

## Special Considerations

### 1. Sample-Accurate Timing
- Max: Implicit sample-accurate timing
- SC: Use `OffsetOut.ar` for sample-accurate playback

### 2. Parameter Smoothing
- Max: Built into many objects
- SC: Use `.lag`, `.varlag`, or `Ramp` UGens

### 3. Preset Management
- Max: pattr system
- SC: Dictionary-based preset storage

### 4. GUI Integration
- Max: Integrated patching/GUI
- SC: Separate GUI system, use Qt or web-based interface

## Testing Mappings

Each mapping should be tested for:
1. **Acoustic equivalence** - Does it sound the same?
2. **Parameter range compatibility** - Do parameters map correctly?
3. **Performance** - Is the SC implementation efficient?
4. **OSC compatibility** - Does it respond to the same messages?

## ATK Integration Examples

### Basic HOA Encoding/Decoding
```supercollider
// Encode a source to HOA
~encoder = FoaEncoderMatrix.newOmni;
~encoded = FoaEncode.ar(source, ~encoder);

// Decode to speaker array
~decoder = FoaDecoderMatrix.newPeri(numChans: 8);
~decoded = FoaDecode.ar(~encoded, ~decoder);
```

### HOA Transformations
```supercollider
// Rotation
~rotated = FoaRotate.ar(~encoded, azimuth);

// Focus/Beamforming
~focused = FoaFocus.ar(~encoded, azimuth, elevation);
```

## Recommendations

1. **Start with basic objects** - Get core audio I/O and routing working first
2. **Implement VBAP early** - It's well-supported in SC and covers many use cases
3. **Use ATK for all HOA** - Don't reinvent the wheel
4. **Preserve OSC namespace** - Makes migration easier
5. **Create abstraction layers** - Hide implementation details from users
6. **Document differences** - Some behaviors may not map exactly
