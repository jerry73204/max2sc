# Max Objects Study and Analysis

This document provides a comprehensive analysis of Max MSP objects found in the analyzed project, with insights for conversion to SuperCollider.

## Analysis Overview

- **Total patches analyzed**: 237
- **Unique object types**: 419
- **Total object instances**: ~20,000+

## Object Categories and Frequency

### Most Common Objects (Top 20)

| Object | Count | Purpose | SC Mapping Strategy |
|--------|-------|---------|---------------------|
| `comment` | 2594 | Documentation | Generate as SC comments |
| `message` | 2509 | Message storage | Constants or Patterns |
| `outlet` | 2261 | Patch outlets | Function returns |
| `inlet` | 2033 | Patch inlets | Function arguments |
| `live.meter~` | 1071 | Level metering | `Amplitude.kr` or `Peak.ar` |
| `bpatcher` | 1006 | Embedded patches | Modular SC functions |
| `adstatus` | 708 | Audio status | `ServerOptions` |
| `thru` | 594 | Pass-through | Direct connection |
| `p` | 532 | Subpatchers | Nested functions |
| `int` | 527 | Integer storage | Variables |
| `live.text` | 452 | UI text | GUI elements (optional) |
| `loadbang` | 354 | Initialization | `s.waitForBoot` blocks |
| `number~` | 322 | Signal monitoring | `Poll.ar` |
| `prepend` | 318 | Message prepend | String operations |
| `live.dial` | 260 | UI control | OSC control mapping |

### Spatial Audio Objects (SPAT5)

#### Most Used SPAT5 Objects
1. **`spat5.osc.route`** (211 instances) - OSC message routing
2. **`spat5.osc.prepend`** (140 instances) - OSC address building
3. **`spat5.viewer`** (95 instances) - Visual feedback
4. **`spat5.osc.routepass`** (95 instances) - OSC pass-through routing
5. **`spat5.osc.view`** (61 instances) - OSC monitoring

#### Key Spatial Processors
- **`spat5.panoramix~`** (6 instances) - Main spatialization engine
- **`spat5.pan~`** (20 instances) - Basic panning
- **`spat5.hoa.encoder~`** (16 instances) - HOA encoding
- **`spat5.hoa.rotate~`** (21 instances) - HOA rotation
- **`spat5.spat~`** (11 instances) - Spatial processor

### Multichannel Objects

| Object | Count | Purpose | SC Conversion |
|--------|-------|---------|---------------|
| `mc.live.gain~` | 56 | Multichannel gain | Array `.collect` with gain |
| `mc.pack~` | 22 | Channel packing | Array construction `[...]` |
| `mc.dac~` | 20 | MC output | `Out.ar` with arrays |
| `mc.unpack~` | 13 | Channel unpacking | Array indexing |
| `mc.adc~` | 7 | MC input | `In.ar` with arrays |
| `mc.combine~` | 3 | Channel combining | Array flattening |

### Audio I/O

| Object | Count | SC Equivalent |
|--------|-------|---------------|
| `dac~` | 26 | `Out.ar` |
| `adc~` | 8 | `In.ar` |
| `ezdac~` | 3 | `{ }.play` |
| `sfplay~` | 3 | `PlayBuf.ar` |

### Signal Processing

| Object | Count | SC Equivalent |
|--------|-------|---------------|
| `*~` | 86 | `*` |
| `+~` | 69 | `+` |
| `/~` | 7 | `/` |
| `cycle~` | 5 | `SinOsc.ar` |
| `noise~` | 1 | `WhiteNoise.ar` |
| `biquad~` | 1 | `SOS.ar` |
| `scale` | 22 | `linlin` or `linexp` |
| `clip` | 10 | `clip` |

## Conversion Complexity Analysis

### Trivial Conversions (Direct Mapping)
These objects have straightforward SC equivalents:
- Basic math operators (`*~`, `+~`, `-~`, `/~`)
- Simple oscillators (`cycle~`, `noise~`)
- Audio I/O (`dac~`, `adc~`)
- Basic routing (`gate~`, `selector~`)

### Moderate Complexity
These require some adaptation:
- `mc.*` objects → Array operations
- `live.*` UI objects → OSC control mapping
- `adstatus` → Server configuration
- Message objects → Pattern system

### High Complexity
These need significant implementation:
- `spat5.panoramix~` → Custom spatial class
- `spat5.hoa.*` → ATK integration
- `bpatcher` → Modular architecture
- `p` (subpatchers) → Nested functions

### Not Directly Convertible
These have no direct SC equivalent:
- UI objects (`live.dial`, `live.meter~`)
- Visual objects (`spat5.viewer`)
- Max-specific (`thispatcher`, `pattr`)

## Object Patterns and Usage

### Initialization Pattern
```
loadbang → message → object
```
Common for setting initial values. Convert to:
```supercollider
s.waitForBoot {
    ~param = initialValue;
};
```

### Multichannel Routing Pattern
```
mc.unpack~ → processing → mc.pack~
```
Convert to SC array operations:
```supercollider
sig = array.collect { |chan| process(chan) };
```

### OSC Control Pattern
```
spat5.osc.route → spat5.osc.prepend → parameter
```
Maintain in SC:
```supercollider
OSCdef(\route, { |msg|
    // Route to parameters
}, '/address/*');
```

## Special Considerations

### Speaker Configurations
The project uses extensive speaker arrays:
- 16, 32, 48, 64 speaker WFS arrays
- Complex routing matrices (128+ channels)
- Per-speaker delay/gain compensation

### High Channel Counts
Many objects handle 64-128 channels:
- `spat5.diagmatrix~ @channels 128`
- `spat5.routing~ @inputs 128 @outputs 128`
- `spat5.sfrecord~ @channels 128`

### Real-time Control
Heavy use of OSC for real-time parameter control:
- 211 instances of `spat5.osc.route`
- Complex OSC namespace
- Parameter interpolation requirements

## Conversion Priorities

### Phase 1: Core Infrastructure
1. Basic audio I/O (`dac~`, `adc~`)
2. Multichannel basics (`mc.pack~`, `mc.unpack~`)
3. Simple routing (`route`, `gate~`)
4. OSC foundation (`spat5.osc.route`)

### Phase 2: Signal Processing
1. Math operations
2. Basic generators
3. Filters and effects
4. Gain control

### Phase 3: Spatial Audio
1. `spat5.pan~` → VBAP
2. `spat5.hoa.encoder~` → ATK
3. `spat5.panoramix~` → Custom implementation
4. Speaker configurations

### Phase 4: Advanced Features
1. Complex routing matrices
2. Snapshot/preset systems
3. Advanced spatial algorithms
4. Performance optimization

## Implementation Notes

### Memory Management
With 128+ channel processing:
- Pre-allocate large bus arrays
- Use efficient buffer management
- Consider block processing

### OSC Efficiency
With 211 OSC routers:
- Implement efficient OSC dispatch
- Use OSC patterns for wildcards
- Cache parameter updates

### Modular Design
With 1006 bpatchers and 532 subpatchers:
- Create reusable SC classes
- Implement clean interfaces
- Support nested architectures

## Recommendations

1. **Start Simple**: Focus on core audio objects first
2. **Preserve Structure**: Maintain Max's modular architecture
3. **Optimize Later**: Get functionality first, optimize after
4. **Test Incrementally**: Validate each object type thoroughly
5. **Document Mappings**: Clear documentation for each conversion