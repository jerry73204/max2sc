# SPAT5 Study and Analysis

This document provides a comprehensive study of the SPAT5 spatial audio framework as found in the analyzed Max8 project.

## Overview

SPAT5 (Spatialisateur) is a comprehensive spatial audio library developed by IRCAM for Max MSP. The analyzed project makes extensive use of SPAT5 for multichannel spatial audio processing.

## Key Findings

### Object Usage Statistics
- **211 instances** of `spat5.osc.route` (primary routing object)
- **140+ unique** SPAT5 object types identified
- **82-channel** multichannel processing capability
- Extensive OSC namespace for control

### Most Common SPAT5 Objects

1. **spat5.osc.route** - OSC message routing and parsing
2. **spat5.panoramix~** - Main spatialization engine
3. **spat5.viewer** - Visual feedback for spatial positions
4. **spat5.osc.prepend** - OSC address manipulation
5. **spat5.hoa.encoder~** - HOA encoding
6. **spat5.hoa.decoder~** - HOA decoding
7. **spat5.reverb~** - Spatial reverberation
8. **spat5.early~** - Early reflections processor

## SPAT5 Architecture

### Core Components

#### 1. Spatialization Engine (panoramix~)
The central object that handles:
- Multiple spatialization algorithms simultaneously
- Source positioning in 3D space
- Distance modeling and air absorption
- Doppler effects
- Multiple bus outputs (direct, early, reverb)

#### 2. OSC Control System
Comprehensive OSC namespace:
```
/source/[n]/xyz         - Cartesian coordinates
/source/[n]/aed         - Spherical coordinates (azimuth, elevation, distance)
/source/[n]/doppler     - Doppler effect amount
/source/[n]/air         - Air absorption
/source/[n]/direct      - Direct sound gain
/source/[n]/early       - Early reflections gain
/source/[n]/reverb      - Reverb send level
```

#### 3. Speaker Configuration
Multiple speaker array support:
- **WFS arrays**: 16, 32, 48, 64 speakers
- **Dome configurations**: Hemispherical setups
- **Custom arrays**: Arbitrary speaker positions
- **Binaural**: Headphone rendering

### Spatial Processing Methods

#### Wave Field Synthesis (WFS)
```
Configuration files found:
- AUO_32speakers_45mm.txt
- AUO_48speakers_45mm.txt  
- AUO_64speakers_45mm.txt
```

Features:
- Pre-delay calculation for wavefront synthesis
- Amplitude compensation
- Anti-aliasing pre-filters
- Support for focused and plane wave sources

#### Higher Order Ambisonics (HOA)
Objects identified:
- `spat5.hoa.encoder~` - Encode sources to HOA
- `spat5.hoa.decoder~` - Decode HOA to speakers
- `spat5.hoa.rotate~` - Rotate sound field
- `spat5.hoa.mirror~` - Mirror transformations
- `spat5.hoa.focus~` - Beamforming
- `spat5.hoa.converter~` - Format conversions

Supports up to 7th order (64 channels).

#### Vector Base Amplitude Panning (VBAP)
- 2D and 3D panning
- Arbitrary speaker configurations
- Distance compensation
- Spread control

### Effects Processing

#### Reverberation
`spat5.reverb~` features:
- Frequency-dependent reverb times
- Early/late reverb balance
- Room size and shape modeling
- Multichannel decorrelation

Parameters:
```
/reverb/tr0     - DC reverb time
/reverb/trl     - Low frequency RT
/reverb/trm     - Mid frequency RT  
/reverb/trh     - High frequency RT
/reverb/decay   - Overall decay
```

#### Early Reflections
`spat5.early~` provides:
- Pattern-based reflections
- Room geometry modeling
- Distance-dependent filtering
- Cluster control

### Utility Objects

#### Metering and Analysis
- `spat5.meter~` - Level metering
- `spat5.rms~` - RMS measurement
- `spat5.scope~` - Oscilloscope
- `spat5.spectroscope~` - Spectrum analyzer

#### Format Conversion
- `spat5.transform~` - Coordinate transformations
- `spat5.converter~` - Format conversions
- `spat5.matrix~` - Routing matrices

## Implementation Patterns

### Typical Signal Flow
```
Input Source
    ↓
spat5.panoramix~
    ├── Direct output → Speaker processing
    ├── Early output → spat5.early~
    └── Reverb output → spat5.reverb~
                            ↓
                     Multichannel mix
```

### OSC Control Architecture
```
OSC Input
    ↓
spat5.osc.route
    ├── /source/* → Source parameters
    ├── /speaker/* → Speaker configuration
    ├── /room/* → Room parameters
    └── /reverb/* → Reverb settings
```

### Multichannel Routing
The project uses complex routing matrices:
- Multiple WFS buses (Bus 1: 16ch, Bus 2: 32ch)
- Reverb buses with independent processing
- Master section with 82-channel capacity

## Conversion Challenges

### 1. Object Complexity
SPAT5 objects encapsulate complex algorithms that require:
- Multiple SC UGens to replicate
- Custom classes for organization
- Careful parameter mapping

### 2. OSC Namespace
The extensive OSC control requires:
- Complete namespace preservation
- Parameter range conversions
- Smooth value interpolation

### 3. Speaker Management
Dynamic speaker configurations need:
- Runtime array reconfiguration
- Efficient delay/gain calculations
- Format-agnostic processing

### 4. Performance
Large channel counts require:
- Efficient DSP chain design
- Optimized buffer usage
- Parallel processing where possible

## Recommended Conversion Strategy

### 1. Modular Architecture
Create SC classes matching SPAT5 structure:
```supercollider
SpatPanoramix {
    var <sources, <speakers;
    var <direct, <early, <reverb;
    
    *ar { |inputs, positions|
        // Modular processing
    }
}
```

### 2. OSC Compatibility Layer
Maintain exact OSC addresses:
```supercollider
OSCdef(\spatOSC, { |msg|
    var address = msg[0];
    var values = msg[1..];
    ~spatOSCRouter.route(address, values);
}, '/spat/*');
```

### 3. Precomputed Lookup Tables
For efficiency:
- Speaker positions and delays
- Panning coefficients
- Filter coefficients

### 4. Test Suite
Comprehensive testing for:
- Spatial accuracy
- OSC response
- CPU performance
- Audio quality

## SPAT5 Features Priority

### High Priority (Core Functionality)
1. spat5.panoramix~ - Essential spatialization
2. spat5.osc.route - Control system
3. Basic HOA encoding/decoding
4. VBAP panning
5. Speaker configuration loading

### Medium Priority (Common Use)
1. spat5.reverb~ - Spatial reverb
2. spat5.early~ - Early reflections
3. WFS processing
4. Distance effects
5. Metering tools

### Low Priority (Advanced)
1. Binaural rendering
2. Advanced HOA transforms
3. Room modeling tools
4. Analysis objects
5. Specialized effects

## Resources and References

### Documentation
- SPAT5 comes with comprehensive Max help files
- Tutorial patches demonstrate each feature
- ADM (Audio Definition Model) support documentation

### Academic Papers
- Various IRCAM publications on spatial audio
- WFS implementation papers
- HOA theory and practice

### Community
- IRCAM Forum users
- Max MSP forums with SPAT5 category
- Academic spatial audio community