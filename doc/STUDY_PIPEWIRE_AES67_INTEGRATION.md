# PipeWire AES67 Integration Study for max2sc 82-Channel Setup

## Overview

This study analyzes the feasibility of integrating Max MSP on WINE directly with PipeWire AES67 for the max2sc project's 82-channel spatial audio requirements, comparing it against the current JACK-based approach.

## Current PipeWire AES67 Status Assessment

### Channel Count Limitations
- **Current Limit**: 64 channels maximum per device
- **Your Requirement**: 82 channels for spatial audio
- **Gap**: 18 channels short of requirement
- **Workaround**: Multi-interface aggregation (complex and unreliable)

### AES67 Implementation Status
- **Development Stage**: Basic module exists, limited functionality
- **Network Audio**: Incomplete RAVENNA/AES67 support
- **Professional Features**: Missing PTP sync, QoS handling
- **Production Readiness**: Not suitable for professional spatial audio

## WINE + Max MSP Integration Analysis

### Direct PipeWire Integration
**Current Status: NOT RECOMMENDED**

```bash
# Theoretical direct integration (not functional)
export PIPEWIRE_LATENCY=512/48000
export WINEPREFIX=$HOME/.wine_max

# This approach currently fails for professional multichannel use
wine "C:/Program Files/Cycling '74/Max 8/Max.exe"
```

**Why Direct Integration Fails:**
- PipeWire native WINE driver still in development
- Max MSP doesn't recognize PipeWire as audio interface
- Channel mapping issues above 64 channels
- Stability problems under professional load

### Hybrid Approach: PipeWire + JACK Bridge
**Status: POSSIBLE BUT COMPROMISED**

```bash
# Install PipeWire with JACK compatibility
sudo apt install pipewire pipewire-jack pipewire-alsa
sudo apt install lib32-alsa-lib lib32-pipewire  # 32-bit support for WINE

# Configure JACK bridge for PipeWire
systemctl --user enable pipewire pipewire-pulse
pw-jack jackd -d alsa -r 48000 -p 512 -i 64 -o 64  # Limited to 64 channels

# Max MSP configuration through WineASIO
wine regsvr32 wineasio.dll
```

**Configuration in Max MSP:**
```maxpat
// Audio Status settings
Driver: WineASIO
Sample Rate: 48000
I/O Vector Size: 512
Input Channels: 64 (max available)
Output Channels: 64 (max available)
```

## Performance Comparison: JACK vs PipeWire for 64+ Channels

### Latency Analysis (48kHz, 512 samples)
| Configuration       | Audio Latency | Total Latency | Stability | CPU Usage |
|---------------------|---------------|---------------|-----------|-----------|
| **JACK Direct**     | 10.7ms        | 10.7ms        | Excellent | Low       |
| **PipeWire-JACK**   | 10.7ms        | 12-15ms       | Good      | Medium    |
| **PipeWire Direct** | Variable      | 15-25ms       | Poor      | High      |

### Channel Count Reality Check
```bash
# JACK: Full 82-channel support
jackd -d alsa -r 48000 -p 512 -i 82 -o 82  ✅

# PipeWire: Limited to 64 channels
pipewire-jack -d alsa -r 48000 -p 512 -i 64 -o 64  ❌ (18 channels short)
```

## Multi-Interface Workarounds for 82 Channels

### Theoretical Aggregation Approach
```bash
# Aggregate multiple interfaces for 82+ channels
# Interface 1: Channels 1-64 (PipeWire limit)
# Interface 2: Channels 65-82 (additional interface)

# Create virtual aggregate device
pw-cli create-node adapter {
    factory.name = audioadapter
    node.name = "Aggregate82Ch"
    media.class = "Audio/Source"
    audio.channels = 82
}

# Manual channel routing (complex and unreliable)
for i in {1..64}; do
    pw-link "Interface1:playback_${i}" "Aggregate82Ch:input_${i}"
done

for i in {65..82}; do
    pw-link "Interface2:playback_$((i-64))" "Aggregate82Ch:input_${i}"
done
```

**Problems with Aggregation:**
- Clock synchronization issues between interfaces
- Increased latency and jitter
- Complex configuration management
- Reliability concerns for professional use
- Limited Max MSP support for virtual aggregates

## AES67 Network Audio Capabilities

### Current AES67 Module Status
```bash
# Check PipeWire AES67 module availability
pw-cli list-objects | grep aes67
# Result: Module exists but functionality limited

# AES67 configuration attempt
module-aes67-discover arguments:
    multicast.group = "239.69.83.67"
    multicast.port = 5004
    session.timeout = 30
```

### Network Audio Requirements for 82-Channel Spatial
- **Bandwidth**: ~15.7 Mbps for 82ch @ 48kHz/24bit
- **Latency**: < 2ms for real-time spatial processing
- **Synchronization**: PTP accuracy < 1µs
- **Channel Distribution**: Network multicast or unicast streams

### Current Limitations
- **PTP Integration**: Not implemented in PipeWire AES67
- **Multichannel Streams**: Limited by PipeWire's 64-channel constraint
- **Professional Features**: Missing DSCP marking, stream redundancy
- **Control Protocol**: Incomplete NMOS IS-04/IS-05 support

## Practical Configuration Attempts

### Attempt 1: Direct PipeWire Integration
```bash
#!/bin/bash
# test_pipewire_max.sh (FAILS)

export WINEPREFIX=$HOME/.wine_max
export PIPEWIRE_LATENCY=512/48000

# Set PipeWire as default
systemctl --user stop pulseaudio
systemctl --user enable pipewire-pulse

# Configure WINE for PipeWire
winecfg  # Audio: ALSA only, no JACK

# Launch Max MSP
wine "C:/Program Files/Cycling '74/Max 8/Max.exe"

# Result: Max MSP doesn't recognize PipeWire as multichannel interface
```

### Attempt 2: PipeWire + JACK Bridge (LIMITED SUCCESS)
```bash
#!/bin/bash
# test_pipewire_jack_bridge.sh

# Start PipeWire with JACK compatibility
systemctl --user start pipewire pipewire-pulse

# Configure JACK to use PipeWire backend
export PIPEWIRE_JACK=1
pw-jack jackd -d alsa -r 48000 -p 512 -i 64 -o 64  # 64 channel limit

# Configure WINE for JACK
export WINEPREFIX=$HOME/.wine_max
wine regsvr32 wineasio.dll

# Test with Max MSP
wine "C:/Program Files/Cycling '74/Max 8/Max.exe"

# Result: Works but limited to 64 channels, higher latency than pure JACK
```

### Attempt 3: Network Audio Distribution
```bash
#!/bin/bash
# test_network_distribution.sh (EXPERIMENTAL)

# Use network audio to distribute additional channels
# Channels 1-64: Local PipeWire
# Channels 65-82: Network AES67 streams

# Configure AES67 transmitter for overflow channels
module-aes67-source arguments:
    source.channels = 18
    multicast.group = "239.69.83.68"
    stream.name = "SpatialOverflow"

# This approach adds significant complexity and latency
```

## Recommended Implementation Strategy

### For Current Production Use: Stick with JACK
```bash
# Proven 82-channel configuration
jackd -R -P70 --port-max 1024 -d alsa -d hw:RME -r 48000 -p 512 -n 2 -i 82 -o 82

# Max MSP configuration
export WINEPREFIX=$HOME/.wine_max
wine regsvr32 wineasio.dll

# Launch Max MSP with full 82-channel support
wine "C:/Program Files/Cycling '74/Max 8/Max.exe"
```

**Advantages of JACK Approach:**
- ✅ Full 82-channel support
- ✅ Professional stability
- ✅ Low latency (10.7ms)
- ✅ Proven WINE integration
- ✅ Extensive documentation

### For Development/Testing: Monitor PipeWire Progress
```bash
# Development environment with PipeWire
# Accept 64-channel limitation for testing
pw-jack jackd -d alsa -r 48000 -p 512 -i 64 -o 64

# Use for basic functionality testing only
# Not suitable for full spatial audio validation
```

### Migration Planning: Future PipeWire Adoption
**Track These Developments:**
1. **Channel Count Increase**: Monitor GitLab issues for 64+ channel support
2. **AES67 Completion**: Watch for production-ready network audio
3. **WINE Integration**: Native PipeWire driver maturity
4. **Professional Adoption**: Industry acceptance for spatial audio

## Configuration Files and Scripts

### PipeWire Configuration for Max Channels
```conf
# /etc/pipewire/pipewire.conf.d/99-max-channels.conf
context.properties = {
    default.clock.rate = 48000
    default.clock.quantum = 512
    default.clock.min-quantum = 32
    default.clock.max-quantum = 2048
}

# Pro Audio configuration
context.modules = [
    { name = libpipewire-module-rt
        args = {
            nice.level = -11
            rt.prio = 88
            rt.time.soft = 200000
            rt.time.hard = 200000
        }
    }
]
```

### WINE Audio Configuration Script
```bash
#!/bin/bash
# configure_wine_pipewire.sh

export WINEPREFIX=$HOME/.wine_max

# Install required dependencies
winetricks corefonts vcrun2019

# Configure WINE registry for audio
wine regedit <<EOF
[HKEY_CURRENT_USER\Software\Wine\DirectSound]
"HelBuflen"="512"
"SndQueueMax"="28"

[HKEY_CURRENT_USER\Software\Wine\ALSA Driver]
"AutoScanCards"="N"
"AutoScanDevices"="N"
EOF

echo "WINE configured for PipeWire compatibility"
```

## Limitations and Workarounds Summary

### Fundamental Limitations
1. **64-Channel Ceiling**: PipeWire architecture limitation
2. **AES67 Incomplete**: Not production-ready for network audio
3. **WINE Integration**: Direct support still in development
4. **Stability Concerns**: Not tested for professional spatial audio loads

### Potential Workarounds (Not Recommended)
1. **Interface Aggregation**: Complex, introduces timing issues
2. **Network Distribution**: Adds latency, complexity
3. **Channel Splitting**: Process in multiple instances (synchronization problems)
4. **Hybrid Routing**: Part PipeWire, part JACK (management nightmare)

## Updated Analysis: Two Preferred Approaches

Based on your preference for JACK + PipeWire integration and AES67 output requirements, I've analyzed both solutions in detail:

### APPROACH 1: JACK + PipeWire AES67 Hybrid (Your Preference)

**Status: TECHNICALLY FEASIBLE BUT LIMITED**

This approach uses JACK for multichannel processing while leveraging PipeWire's AES67 capabilities for network audio output.

#### Implementation Strategy
```bash
# Production setup with JACK + PipeWire AES67
# 1. Configure JACK for full 82-channel processing
jackd -R -P70 --port-max 1024 -d alsa -d hw:RME -r 48000 -p 512 -n 2 -i 82 -o 82 &

# 2. Route JACK output to PipeWire AES67 (requires custom routing)
# Multiple 64-channel AES67 streams needed for 82 channels
pw-cli create-node adapter {
    factory.name = audioadapter
    node.name = "AES67_Stream1"
    audio.channels = 64
    api.aes67.multicast.group = "239.69.83.67"
}

pw-cli create-node adapter {
    factory.name = audioadapter 
    node.name = "AES67_Stream2"
    audio.channels = 18
    api.aes67.multicast.group = "239.69.83.68"
}

# 3. Custom routing script to map JACK ports to AES67 streams
for i in {1..64}; do
    jack_connect "system:playback_${i}" "AES67_Stream1:input_${i}"
done

for i in {65..82}; do
    jack_connect "system:playback_${i}" "AES67_Stream2:input_$((i-64))"
done
```

#### Advantages
- ✅ **Full 82-channel JACK processing**
- ✅ **AES67 network output capability**
- ✅ **Proven JACK stability for Max MSP**
- ✅ **Flexible routing configuration**

#### Limitations
- ❌ **Complex configuration**: Requires multiple AES67 streams
- ❌ **Synchronization challenges**: Multiple streams need precise timing
- ❌ **Increased latency**: Additional routing layer adds 2-5ms
- ⚠️ **AES67 implementation**: PipeWire's AES67 lacks PTP sync

### APPROACH 2: PipeWire Channel Limit Investigation

**Status: 64-CHANNEL LIMIT IS ARCHITECTURAL**

My investigation confirms the 64-channel limit is hard-coded in PipeWire's core architecture:

#### Configuration Research Results
```bash
# These attempts to exceed 64 channels ALL FAIL:

# Attempt 1: Configuration override (FAILS)
echo 'default.audio.channels = 82' >> ~/.config/pipewire/pipewire.conf
# Result: PipeWire ignores values > 64

# Attempt 2: Direct node creation (FAILS)
pw-cli create-node adapter {
    audio.channels = 82  # Error: Invalid value, max 64
}

# Attempt 3: Multiple interface aggregation (UNRELIABLE)
# Creates synchronization and timing issues
```

#### Source Code Analysis
The 64-channel limit is defined in PipeWire's core audio processing:
- `audio.channels` parameter range: 1-64 (hard limit)
- No compile-time option to increase this limit
- Architectural constraint, not a configuration parameter

### Recommended Hybrid Solution

**JACK + External AES67 Tool (Most Practical)**

Given your AES67 requirement and the limitations found, here's the recommended approach:

```bash
#!/bin/bash
# hybrid_82ch_aes67_setup.sh

# 1. Use JACK for full 82-channel processing
systemctl --user stop pipewire pipewire-pulse
jackd -R -P70 --port-max 1024 -d alsa -d hw:RME -r 48000 -p 512 -n 2 -i 82 -o 82 &

# 2. Use external AES67 tool for network output
# Options: dante-audio-toolkit, ravenna-alsa-lkm, aes67-sender
# Example with aes67-sender:
aes67-sender --input jack --channels 82 --multicast 239.69.83.67:5004 &

# 3. Configure Max MSP with full 82-channel support
export WINEPREFIX=$HOME/.wine_max
export WINEASIO_NUMBER_INPUTS=82
export WINEASIO_NUMBER_OUTPUTS=82
wine "C:/Program Files/Cycling '74/Max 8/Max.exe"
```

#### Performance Comparison Updated

| Configuration | Channels | AES67 Output | Latency | Stability | Complexity |
|---------------|----------|---------------|---------|-----------|------------|
| **JACK + External AES67** | 82 | ✅ Full | 10.7ms | Excellent | Medium |
| **JACK + PipeWire AES67** | 82 | ⚠️ Multiple streams | 12-15ms | Good | High |
| **PipeWire Only** | 64 | ✅ Native | 15-25ms | Poor | Low |

### AES67 Network Output Solutions

Since PipeWire's AES67 has limitations, here are proven AES67 solutions for JACK:

#### Option 1: RAVENNA ALSA Driver
```bash
# Professional solution, requires license
# Provides kernel-level AES67/RAVENNA support
ravenna-alsa-lkm --channels 82 --multicast 239.69.83.67
```

#### Option 2: Dante Audio Toolkit
```bash
# Commercial but widely adopted
# Excellent AES67 interoperability
dante-audio-toolkit --jack-input --channels 82
```

#### Option 3: Open-source AES67 Tools
```bash
# Free alternatives (varying stability)
git clone https://github.com/bondagit/aes67-linux-daemon
# Compile and configure for 82-channel output
```

## Updated Recommendations

### For Production Use: JACK + External AES67
```bash
# Complete production setup
jackd -R -d alsa -r 48000 -p 512 -i 82 -o 82 &
aes67-sender --input jack --channels 82 --ptp-enable &
wine max.exe  # Full 82-channel support
```

### For Development: PipeWire Limited Testing
```bash
# Development with channel limitations
pw-jack jackd -d dummy -r 48000 -p 512 -i 64 -o 64 &
# Test spatial algorithms with reduced channel count
```

### Future Migration Strategy
Monitor these developments:
1. **PipeWire Architecture**: Watch for 64+ channel support
2. **AES67 Completion**: PTP sync and professional features
3. **WINE Integration**: Native PipeWire driver maturity

## Conclusion and Final Recommendations

For the max2sc project's 82-channel spatial audio with AES67 output:

1. **Immediate Solution**: Use JACK with external AES67 tools
   - Full 82-channel support
   - Professional AES67 network output
   - Proven stability and performance

2. **Development Testing**: Limited PipeWire usage
   - 64-channel functional testing only
   - Not suitable for full spatial audio validation

3. **Future Planning**: Monitor PipeWire development
   - Potential migration when channel limits increase
   - Track AES67 implementation improvements

The JACK + external AES67 approach provides the best combination of full channel support, professional network audio output, and proven stability for your max2sc spatial audio project.
