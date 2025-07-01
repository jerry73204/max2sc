# Conversion Strategy Design Decisions

This document outlines the design decisions for converting Max MSP patches to SuperCollider code.

## Object Mapping Strategy

### Direct vs. Semantic Mapping

**Decision**: Use semantic mapping over direct 1:1 object translation.

**Rationale**:
- Max and SC have different paradigms (visual patching vs. code)
- Semantic mapping produces more idiomatic SC code
- Better performance and maintainability
- Preserves intent rather than implementation

**Example**:
```
Max: [sig~ 440] → [cycle~] → [*~ 0.5] → [dac~]
SC:  { SinOsc.ar(440) * 0.5 }.play  // More idiomatic
```

### Multichannel Handling

**Decision**: Convert Max's mc.* objects to SC array operations.

**Mapping Strategy**:
- `mc.pack~` → Array construction `[sig1, sig2, ...]`
- `mc.unpack~` → Array indexing `array[0..n]`
- `mc.dup~` → `.dup(n)` or `!n`
- `mc.combine~` → Array flattening operations

**Rationale**:
- SC's array operations are more flexible
- Better performance with native array handling
- Cleaner code generation

### Signal Flow Analysis

**Decision**: Build complete signal flow graph before conversion.

**Benefits**:
1. **Optimization**: Identify redundant operations
2. **Ordering**: Ensure correct execution order
3. **Grouping**: Bundle related operations
4. **Analysis**: Detect patterns for better conversion

**Implementation**:
```rust
pub struct SignalPath {
    pub source: NodeIndex,
    pub processors: Vec<NodeIndex>,
    pub destination: NodeIndex,
    pub path_type: PathType, // Audio, Control, Message
}
```

## Code Generation Strategy

### SynthDef vs. Function Approach

**Decision**: Generate SynthDefs for reusable components, functions for one-offs.

**Guidelines**:
- **SynthDef**: Spatial processors, instruments, effects chains
- **Function**: Simple calculations, initialization code
- **Hybrid**: Main patch logic with SynthDef building blocks

**Example**:
```supercollider
// Reusable spatial processor as SynthDef
SynthDef(\spatPanoramix8, { |bus, azimuth = 0|
    var input = In.ar(bus);
    var panned = PanAz.ar(8, input, azimuth.lag(0.02));
    Out.ar(0, panned);
}).add;

// One-off initialization as function
~initializeProject = {
    ~buses = 32.collect { Bus.audio(s, 1) };
    ~groups = ();
    // ...
};
```

### Parameter Management

**Decision**: Use a three-tier parameter system.

**Tiers**:
1. **Static Parameters**: Compile-time constants
2. **Dynamic Parameters**: Runtime controllable via args
3. **Modulated Parameters**: Audio-rate or control-rate modulation

**Implementation**:
```supercollider
SynthDef(\example, {
    |freq = 440,           // Dynamic parameter
     amp = 0.5|            // Dynamic parameter
    
    var modDepth = 0.1;    // Static parameter
    var mod = SinOsc.kr(5) * modDepth;
    var sig = SinOsc.ar(freq + (freq * mod)) * amp;
    Out.ar(0, sig);
});
```

### Bus Architecture

**Decision**: Generate comprehensive bus routing system.

**Bus Types**:
```supercollider
~buses = (
    // Audio buses for sources
    sources: 32.collect { Bus.audio(s, 1) },
    
    // Multichannel buses for spatial processing
    spatial: [
        Bus.audio(s, 8),  // 8-channel
        Bus.audio(s, 16), // 16-channel
        Bus.audio(s, 32)  // 32-channel
    ],
    
    // Effect sends
    effects: (
        reverb: Bus.audio(s, 2),
        delay: Bus.audio(s, 2),
        early: Bus.audio(s, 8)
    ),
    
    // Control buses for parameters
    control: (
        azimuth: Bus.control(s, 32),
        elevation: Bus.control(s, 32),
        distance: Bus.control(s, 32)
    )
);
```

**Rationale**:
- Matches Max's flexible routing
- Enables complex signal paths
- Efficient resource usage

## Pattern and Control Flow

### Message vs. Signal Rate

**Decision**: Analyze and preserve Max's timing semantics.

**Conversion Rules**:
1. **Audio connections** → Audio rate in SC
2. **Control connections** → Control rate or patterns
3. **Message connections** → Immediate or scheduled events

**Example**:
```supercollider
// Max metro → SC pattern
~metro = Routine {
    loop {
        ~trigger.value;
        0.5.wait;  // 500ms interval
    }
}.play;

// Max line~ → SC envelope
Env([0, 1], [2], \lin).ar;
```

### Preset and Snapshot System

**Decision**: Implement dictionary-based preset system.

**Structure**:
```supercollider
~presets = IdentityDictionary[
    \default -> (
        azimuth: 0,
        elevation: 0,
        reverb: 0.3,
        // ...
    ),
    \preset1 -> (
        azimuth: 45,
        elevation: 15,
        reverb: 0.7,
        // ...
    )
];

~loadPreset = { |name|
    var preset = ~presets[name];
    preset.keysValuesDo { |param, value|
        ~setParameter.value(param, value);
    };
};
```

**Features**:
- Interpolation between presets
- Save/load to files
- OSC control compatibility

## Error Handling and Validation

### Graceful Degradation

**Decision**: Generate code that handles missing resources gracefully.

**Strategies**:
1. **Missing audio files**: Use silent input or placeholder
2. **Invalid parameters**: Clamp to valid ranges
3. **Missing plugins**: Provide fallback implementations
4. **Speaker config errors**: Default to standard setup

**Example**:
```supercollider
// Graceful file loading
~loadBuffer = { |path|
    if(File.exists(path), {
        Buffer.read(s, path)
    }, {
        warn("File not found: " ++ path);
        Buffer.alloc(s, 44100, 1); // 1-second silent buffer
    })
};
```

### Parameter Validation

**Decision**: Generate parameter validation code.

**Implementation**:
```supercollider
~setAzimuth = { |source, azimuth|
    // Validate and wrap azimuth to -180..180
    azimuth = azimuth.wrap(-180, 180);
    ~sources[source].set(\azimuth, azimuth);
};

~setSpeakerCount = { |count|
    // Validate speaker count
    count = count.clip(2, 64);
    ~rebuildSpeakerArray.value(count);
};
```

## Optimization Strategies

### Node Graph Optimization

**Decision**: Optimize before code generation.

**Optimizations**:
1. **Constant folding**: Pre-calculate static values
2. **Dead code elimination**: Remove unused branches
3. **Common subexpression**: Share repeated calculations
4. **Strength reduction**: Use simpler operations

### Memory Management

**Decision**: Generate efficient buffer and bus allocation.

**Strategies**:
```supercollider
// Pre-allocate buffers
~buffers = (
    impulses: 16.collect { Buffer.alloc(s, 2048, 1) },
    delays: 32.collect { Buffer.alloc(s, s.sampleRate, 1) },
    tables: 8.collect { Buffer.alloc(s, 512, 1) }
);

// Reuse temporary buffers
~tempBuffers = 4.collect { Buffer.alloc(s, 65536, 1) };
~getTempBuffer = { ~tempBuffers.detect(_.numFrames > 0) };
```

## Testing Strategy

### Generated Code Validation

**Decision**: Include self-test code in generated projects.

**Test Categories**:
1. **Syntax validation**: Code compiles without errors
2. **Resource validation**: All buses/buffers allocated
3. **Signal flow validation**: Audio reaches outputs
4. **OSC validation**: All addresses respond

**Example**:
```supercollider
~runTests = {
    "Running project tests...".postln;
    
    // Test SynthDef compilation
    ~synthDefs.do { |def|
        try {
            def.add;
            ("✓ SynthDef compiled: " ++ def.name).postln;
        } { |error|
            ("✗ SynthDef failed: " ++ def.name).postln;
            error.throw;
        }
    };
    
    // Test bus allocation
    ~buses.keysValuesDo { |key, bus|
        if(bus.index.notNil) {
            ("✓ Bus allocated: " ++ key).postln;
        } {
            ("✗ Bus failed: " ++ key).postln;
        }
    };
    
    "All tests passed!".postln;
};
```

## Documentation Generation

### Inline Documentation

**Decision**: Generate comprehensive inline documentation.

**Documentation Includes**:
- Original Max object mappings
- Parameter descriptions
- Signal flow comments
- Usage examples

**Format**:
```supercollider
/*
 * SpatPanoramix8 - 8-channel spatial panner
 * 
 * Converted from: spat5.panoramix~ @speakers 8
 * 
 * Parameters:
 *   - azimuth: -180 to 180 degrees
 *   - elevation: -90 to 90 degrees  
 *   - distance: 0 to inf meters
 *
 * OSC Control:
 *   /source/1/azimuth <float>
 *   /source/1/elevation <float>
 *   /source/1/distance <float>
 */
```

### Project Documentation

**Decision**: Auto-generate README with project overview.

**Contents**:
1. Project structure
2. Requirements and dependencies
3. Usage instructions
4. OSC namespace reference
5. Troubleshooting guide

## Future-Proofing

### Version Compatibility

**Decision**: Generate version-aware code.

**Implementation**:
```supercollider
// Version check
if(Main.versionAtLeast(3, 11), {
    // Use newer features
    ~reverb = JPverb.ar(input);
}, {
    // Fall back to older implementation
    ~reverb = GVerb.ar(input);
});
```

### Extension Points

**Decision**: Design generated code for easy modification.

**Features**:
- Clear hook points for customization
- Modular structure
- Configuration files for tweaking
- Well-defined APIs for extensions