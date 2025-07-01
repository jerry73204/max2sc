# Spatial Audio Design Decisions

This document details the design decisions specifically related to spatial audio processing in the max2sc converter.

## Core Spatial Architecture

### Modular Spatialization Pipeline

**Decision**: Implement spatial processing as a three-stage pipeline:
```
Source → Encoder → Processor → Decoder → Output
```

**Rationale**:
- Matches SPAT5's internal architecture
- Allows mixing different spatial formats (e.g., encode to HOA, process, decode to VBAP)
- Easier to extend with new spatial methods
- Clear separation of concerns

**Implementation**:
```supercollider
// Example: HOA to VBAP pipeline
~source = In.ar(bus);
~encoded = HoaEncode.ar(~source, ~encoder);
~processed = HoaRotate.ar(~encoded, azimuth, elevation);
~decoded = VBAPDecoder.ar(~processed, ~speakerArray);
```

## Spatial Method Mappings

### SPAT5 Panoramix

**Decision**: Create a comprehensive `SpatPanoramix` class that encapsulates all SPAT5 panoramix functionality.

**Components**:
1. **Direct Sound Path**: Primary spatial positioning
2. **Early Reflections**: Room-based early reflection patterns
3. **Reverb Send**: Configurable reverb processing
4. **Distance Effects**: Air absorption, Doppler, attenuation

**Rationale**:
- SPAT5 panoramix is the most complex and frequently used spatial object
- Modular design allows using only needed components
- Maintains compatibility with existing OSC control

### Wave Field Synthesis (WFS)

**Decision**: Implement WFS as a flexible array processor supporting multiple array geometries.

**Array Types Supported**:
- Linear arrays (most common for WFS)
- Circular arrays
- Irregular/custom arrays

**Key Features**:
```supercollider
WFSArray {
    var <speakers;          // Speaker positions
    var <delayTimes;       // Per-speaker delays
    var <amplitudes;       // Per-speaker gains
    var <prefilters;       // Anti-aliasing filters
    
    *new { |geometry, spacing = 0.045| }
    
    synthesizePointSource { |x, y, z| }
    synthesizePlaneWave { |azimuth| }
    synthesizeFocusedSource { |x, y, z, focus| }
}
```

**Rationale**:
- WFS requires precise delay and amplitude calculations
- Pre-calculation improves real-time performance
- Different source types need different synthesis methods

### Vector Base Amplitude Panning (VBAP)

**Decision**: Use SuperCollider's built-in VBAP UGen with custom speaker management.

**Speaker Configuration**:
```supercollider
VBAPSpeakerArray {
    var <positions;      // [[azi1, ele1], [azi2, ele2], ...]
    var <triangulation;  // Pre-calculated for 3D
    
    *ring { |n = 8, elevation = 0| }
    *dome { |rings, elevations| }
    *custom { |positions| }
}
```

**Rationale**:
- SC's VBAP is well-tested and efficient
- Custom speaker array management provides flexibility
- Pre-calculated triangulation improves performance

### Higher Order Ambisonics (HOA)

**Decision**: Full integration with Ambisonic Toolkit (ATK) for SuperCollider.

**Order Support**:
- 1st order (4 channels) - FOA
- 2nd order (9 channels)
- 3rd order (16 channels)
- Up to 7th order (64 channels)

**Conversion Strategy**:
```
spat5.hoa.encoder~ → FoaEncode / HoaEncodeMatrix
spat5.hoa.decoder~ → FoaDecode / HoaDecodeMatrix
spat5.hoa.rotate~  → FoaRotate / HoaRotate
spat5.hoa.focus~   → FoaFocus / HoaFocus
```

**Rationale**:
- ATK is the de facto standard for ambisonics in SC
- Comprehensive feature set matches SPAT5 capabilities
- Active development and community support

## Distance and Room Effects

### Distance-Based Processing

**Decision**: Implement as a modular `DistanceProcessor` with configurable components.

**Components**:
1. **Attenuation**: Multiple models (inverse, inverse square, custom)
2. **Air Absorption**: Frequency-dependent filtering based on distance
3. **Doppler Effect**: Pitch shifting based on source movement
4. **Initial Time Gap**: Delay before first reflection

**Implementation**:
```supercollider
DistanceProcessor {
    var <attenuationModel;
    var <airAbsorptionCutoff;
    var <dopplerAmount;
    
    *ar { |input, distance, velocity|
        var attenuated = this.attenuate(input, distance);
        var absorbed = this.airAbsorb(attenuated, distance);
        var doppler = this.doppler(absorbed, velocity);
        ^doppler
    }
}
```

### Early Reflections

**Decision**: Pattern-based early reflection generator with room presets.

**Room Models**:
- Shoebox (rectangular room)
- Concert hall (fan-shaped)
- Cathedral (high ceiling, long reverb)
- Custom (user-defined)

**Implementation Approach**:
- Tapped delay lines for efficiency
- Pattern generation based on room geometry
- Distance-based filtering for each reflection

### Reverberation

**Decision**: Wrapper system around SC reverb UGens with parameter mapping.

**Supported Reverbs**:
- `JPverb` - High quality, flexible
- `GVerb` - Classic Schroeder reverb
- `FreeVerb` - Lightweight option
- Custom reverb networks

**Parameter Mapping**:
```
SPAT5 Parameters → SC Parameters
tr0 (DC reverb time) → roomsize
trl (low freq RT) → damping + custom filter
trm (mid freq RT) → roomsize
trh (high freq RT) → damping
```

## Speaker Configuration Management

### Configuration File Format

**Decision**: Parse Max's OSC-style configuration files directly.

**Format Support**:
```
/speaker/1/xyz -0.500 0.000 0.000
/speaker/1/aed 180.0 0.0 0.500
/speaker/1/delay 0.00145
/speaker/1/gain 1.0
```

**Rationale**:
- Maintains compatibility with existing speaker configs
- No manual conversion needed
- Preserves calibration data

### Dynamic Speaker Arrays

**Decision**: Support runtime reconfiguration of speaker arrays.

**Features**:
- Hot-swapping speaker configurations
- Smooth transitions between configs
- Automatic re-calculation of spatial parameters

## OSC Control Protocol

### Namespace Preservation

**Decision**: Maintain exact OSC namespace compatibility with Max/SPAT5.

**Key Namespaces**:
```
/source/[n]/*     - Source positioning and parameters
/speaker/[n]/*    - Speaker configuration
/room/*           - Room acoustics parameters
/reverb/[n]/*     - Reverb settings
/bus/[n]/*        - Bus routing and processing
/master/*         - Master section controls
```

**Implementation**:
```supercollider
OSCdef(\sourceAzimuth, { |msg|
    var sourceIndex = msg[0].asString.split($/).at(2).asInteger;
    var azimuth = msg[1];
    ~sources[sourceIndex].set(\azimuth, azimuth);
}, '/source/*/azimuth');
```

### Parameter Smoothing

**Decision**: Automatic parameter smoothing with configurable ramp times.

**Approach**:
- Default 20ms ramp for position changes
- Configurable per-parameter ramp times
- Block-rate updates for efficiency

## Performance Optimizations

### Pre-calculation Strategy

**Decision**: Pre-calculate static values during initialization.

**Pre-calculated Data**:
- Speaker positions and delays
- VBAP triangulation
- WFS array geometry
- HOA decoder matrices

**Rationale**:
- Reduces real-time CPU load
- Improves accuracy (more time for complex calculations)
- Enables larger speaker arrays

### Efficient Multi-source Processing

**Decision**: Use parallel processing for multiple sources.

**Implementation**:
```supercollider
// Process multiple sources in parallel
~sources = 32.collect { |i|
    { 
        var input = In.ar(~sourceBuses[i]);
        SpatialProcessor.ar(input, ~positions[i])
    }.play(target: ~spatialGroup)
};
```

## Testing and Validation

### Spatial Accuracy Testing

**Decision**: Implement automated spatial accuracy tests.

**Test Methods**:
1. **Phantom Source Location**: Verify perceived vs intended position
2. **Energy Preservation**: Check total energy remains constant
3. **Smooth Transitions**: No clicks/artifacts during movement
4. **Phase Coherence**: Maintain phase relationships

### A/B Testing Framework

**Decision**: Create tools for comparing Max and SC output.

**Components**:
- Synchronized playback system
- Spectral analysis comparison
- Perceptual metrics
- Automated report generation

## Future Extensibility

### Plugin Architecture for Spatial Methods

**Decision**: Design for easy addition of new spatial algorithms.

**Extension Points**:
1. Custom encoder/decoder pairs
2. New distance models
3. Alternative room simulations
4. Novel spatial effects

### Machine Learning Integration

**Decision**: Prepare for ML-based spatial processing.

**Considerations**:
- Neural reverb models
- Learned HRTF interpolation
- Automatic speaker array optimization
- Perceptual spatial enhancement