# Setting up Max MSP / SuperCollider Comparison Testing

## Overview

This guide explains how to set up a comparison testing environment where Max MSP runs on WINE and SuperCollider runs natively on Linux to validate max2sc conversions.

## Prerequisites

### System Requirements
- Linux system (Ubuntu 20.04+ recommended)
- At least 8GB RAM
- Audio interface or virtual audio setup
- WINE 6.0+
- SuperCollider 3.12+

### Software Installation

#### 1. Install WINE
```bash
# Add WINE repository
sudo dpkg --add-architecture i386
wget -qO - https://dl.winehq.org/wine-builds/winehq.key | sudo apt-key add -
sudo add-apt-repository 'deb https://dl.winehq.org/wine-builds/ubuntu/ focal main'

# Install WINE
sudo apt update
sudo apt install --install-recommends winehq-stable winetricks

# Verify installation
wine --version
```

#### 2. Configure WINE for Max MSP
```bash
# Create dedicated WINE prefix for Max
export WINEPREFIX=$HOME/.wine_max
export WINEARCH=win64

# Initialize WINE prefix
winecfg

# Install dependencies
winetricks corefonts vcrun2019 dotnet48

# Configure audio (choose ALSA or PulseAudio)
winecfg  # Audio tab -> ALSA Out or PulseAudio
```

#### 3. Install Max MSP in WINE
```bash
# Download Max MSP installer (requires license)
# Run installer in WINE
wine MaxInstaller.exe

# Test Max MSP installation
wine "C:/Program Files/Cycling '74/Max 8/Max.exe" -version
```

#### 4. Install SuperCollider
```bash
# Install SuperCollider
sudo apt install supercollider supercollider-headless

# Install additional packages if needed
sudo apt install sc3-plugins supercollider-extensions

# Verify installation
sclang -v
```

## Audio System Setup - 64-Channel Analysis

### Option 1: JACK Audio (Recommended for 64+ Channels)

**Channel Capabilities:**
- ✅ **Supports 64+ channels**: JACK can handle as many channels as hardware supports
- ✅ **Configurable port limit**: Default 256 ports, expandable with `--port-max`
- ✅ **Production ready**: Designed for professional multichannel audio
- ✅ **Real-time performance**: Low-latency operation with proper configuration

```bash
# Install JACK
sudo apt install jackd2 qjackctl

# Configure system for real-time audio
sudo usermod -a -G audio $USER
echo "@audio - rtprio 99" | sudo tee -a /etc/security/limits.conf
echo "@audio - memlock unlimited" | sudo tee -a /etc/security/limits.conf

# Start JACK for 64-channel operation
jackd -R -P70 --port-max 512 -d alsa -d hw:0 -r 48000 -p 512 -n 2 -i 64 -o 64

# For max2sc 82-channel requirement
jackd -R -P70 --port-max 512 -d alsa -d hw:0 -r 48000 -p 512 -n 2 -i 82 -o 82

# Configure SuperCollider for JACK 64-channel
cat >> ~/.config/SuperCollider/startup.scd << 'EOF'
Server.default = Server.local;
Server.default.options.device = "JackRouter";
Server.default.options.numOutputBusChannels = 64;
Server.default.options.numInputBusChannels = 64;
Server.default.options.blockSize = 512;  // Match JACK buffer
EOF
```

**Performance Considerations for 64 Channels:**
- **CPU Usage**: Scales linearly with channel count (64x stereo load)
- **Buffer Size**: 512-1024 samples recommended for stability
- **Latency**: ~10.7ms @ 48kHz with 512 sample buffer
- **Memory**: Significant bandwidth requirements for 64-channel streams

**Hardware Requirements:**
```bash
# Professional multichannel interfaces
# - RME HDSP series (up to 64+ channels)
# - MOTU AVB interfaces (64+ channels)
# - Multiple aggregated USB interfaces

# Verify hardware channel count
aplay -l  # List playback devices
arecord -l  # List capture devices
```

### Option 2: snd-dummy (Limited for 64-Channel Use)

**Channel Limitations:**
- ❌ **Not designed for 64-channel streams**: snd-dummy supports multiple substreams, not 64 channels per stream
- ❌ **No actual I/O**: Virtual driver for testing only
- ❌ **Performance concerns**: Not optimized for high channel counts
- ⚠️ **Use case**: Basic testing only, not production validation

```bash
# Install dummy audio (not recommended for 64-channel)
sudo modprobe snd-dummy pcm_substreams=64

# Limited SuperCollider configuration
echo 'Server.default.options.device = "hw:Dummy,0";' >> ~/.config/SuperCollider/startup.scd
# Note: This will NOT provide true 64-channel capability
```

### Option 3: PipeWire (Modern Alternative)

**64-Channel Capabilities:**
- ✅ **JACK compatibility**: Drop-in replacement with JACK client support
- ✅ **Unified audio**: Replaces both PulseAudio and JACK
- ✅ **Multichannel support**: Native support for high channel counts
- ⚠️ **Maturity**: Newer system, less field-tested than JACK

```bash
# Install PipeWire
sudo apt install pipewire pipewire-jack pipewire-alsa

# Enable PipeWire services
systemctl --user enable pipewire pipewire-pulse

# Configure for 64-channel operation
# Edit /etc/pipewire/pipewire.conf for multichannel setup
```

### Recommended Setup for max2sc 82-Channel Testing

**Production Setup (Hardware Available):**
```bash
# JACK with professional interface
jackd -R -P70 --port-max 1024 -d alsa -d hw:RME -r 48000 -p 512 -n 2 -i 82 -o 82

# SuperCollider configuration
s.options.numOutputBusChannels = 82;
s.options.numInputBusChannels = 82;
s.options.blockSize = 512;
```

**Development/CI Setup (No Hardware):**
```bash
# Use snd-aloop for loopback testing (better than snd-dummy)
sudo modprobe snd-aloop pcm_substreams=8 index=1

# Create virtual 82-channel interface using multiple loops
# Configure JACK to use loopback interfaces
jackd -d alsa -d hw:Loopback,0,0 -r 48000 -p 1024 -n 2 -i 82 -o 82
```

**Testing and Validation:**
```bash
# Test JACK 64-channel setup
jack_simple_client  # Basic connection test
jack_lsp -c  # List all ports and connections
jack_cpu_load  # Monitor CPU usage and xruns

# SuperCollider channel test
echo "{ Out.ar((0..63), SinOsc.ar((100..163))) }.play;" | sclang
```

## Environment Configuration

### 1. Set Environment Variables
```bash
# Add to ~/.bashrc or ~/.profile
export WINEPREFIX=$HOME/.wine_max
export MAX_MSP_PATH="$WINEPREFIX/drive_c/Program Files/Cycling '74/Max 8/Max.exe"
export SCLANG_PATH=/usr/bin/sclang

# For headless testing
export QT_QPA_PLATFORM=offscreen
export DISPLAY=:99  # If using Xvfb
```

### 2. Create Test Scripts

#### WINE Test Script
```bash
#!/bin/bash
# test_max_wine.sh

export WINEPREFIX=$HOME/.wine_max
export WINEDEBUG=-all

# Start virtual display for headless mode
Xvfb :99 -screen 0 1024x768x24 &
export DISPLAY=:99

# Run Max MSP patch
wine "$MAX_MSP_PATH" -nogui -nosplash "$1"

# Cleanup
killall Xvfb
```

#### SuperCollider Test Script
```bash
#!/bin/bash
# test_sc_native.sh

# Set headless mode
export QT_QPA_PLATFORM=offscreen

# Run SuperCollider code
sclang -i none "$1"
```

## Comparison Testing Setup

### 1. Install max2sc with Comparison Features
```bash
# Add to Cargo.toml
[features]
comparison-tests = ["max2sc-test/comparison"]

# Build with comparison support
cargo build --features comparison-tests
```

### 2. Create Test Data Structure
```
tests/comparison/
├── patches/                 # Max MSP patches
│   ├── sine_440.maxpat
│   ├── vbap_8ch.maxpat
│   └── hoa_3rd_order.maxpat
├── reference/              # Expected audio outputs
│   ├── sine_440_ref.wav
│   ├── vbap_8ch_ref.wav
│   └── hoa_3rd_order_ref.wav
├── tolerances.json         # Comparison tolerances
└── test_config.toml        # Test configuration
```

### 3. Configure Comparison Tolerances
```json
{
  "audio": {
    "rms_tolerance": 0.02,
    "spectral_similarity": 0.95,
    "peak_tolerance": 0.05
  },
  "timing": {
    "onset_tolerance_ms": 10,
    "duration_tolerance_ms": 50
  },
  "spatial": {
    "channel_balance_tolerance": 0.1,
    "localization_accuracy": 0.9
  }
}
```

## Running Comparison Tests

### 1. Manual Testing
```bash
# Test individual conversion
cargo run --bin max2sc-compare \
  --max-patch tests/patches/sine_440.maxpat \
  --sc-code "SynthDef(\\sine, { Out.ar(0, SinOsc.ar(440) * 0.5) }).add; Synth(\\sine);" \
  --duration 5 \
  --output-dir /tmp/comparison

# View results
cat /tmp/comparison/report.md
```

### 2. Automated Test Suite
```bash
# Run all comparison tests
cargo test --features comparison-tests comparison::

# Generate detailed report
cargo test --features comparison-tests comparison:: -- --nocapture > comparison_results.log
```

### 3. CI/CD Integration
```yaml
# .github/workflows/comparison.yml
name: Max/SC Comparison Tests

on: [push, pull_request]

jobs:
  comparison:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup environment
        run: |
          sudo apt-get update
          sudo apt-get install -y wine supercollider-headless xvfb
          
      - name: Configure WINE
        run: |
          export WINEPREFIX=$HOME/.wine_max
          winecfg /v  # Configure automatically
          
      - name: Run comparison tests
        run: |
          Xvfb :99 -screen 0 1024x768x24 &
          export DISPLAY=:99
          cargo test --features comparison-tests
          
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: comparison-results
          path: target/comparison_reports/
```

## Troubleshooting

### Common Issues

#### WINE Audio Problems
```bash
# Check WINE audio configuration
winecfg  # Audio tab

# Test WINE audio
wine notepad.exe  # Should produce system sounds
```

#### SuperCollider Server Issues
```bash
# Check SC server status
echo "s.boot; s.quit;" | sclang

# Test audio output
echo "{ SinOsc.ar(440) * 0.1 }.play;" | sclang
```

#### Permission Errors
```bash
# Add user to audio group
sudo usermod -a -G audio $USER

# Restart session or run
newgrp audio
```

### Performance Optimization

#### Reduce WINE Overhead
```bash
# Disable unnecessary WINE features
export WINEDEBUG=-all
export WINE_CPU_TOPOLOGY=4:2  # Limit CPU cores

# Use minimal WINE prefix
rm -rf $WINEPREFIX/.wine/drive_c/windows/Fonts/*  # Remove unused fonts
```

#### Optimize SuperCollider
```bash
# Use optimized server options
echo 'Server.default.options.blockSize = 256;' >> ~/.config/SuperCollider/startup.scd
echo 'Server.default.options.sampleRate = 44100;' >> ~/.config/SuperCollider/startup.scd
echo 'Server.default.options.memSize = 8192;' >> ~/.config/SuperCollider/startup.scd
```

## Best Practices

### 1. Test Design
- Use deterministic patches (avoid random elements)
- Test short duration samples (5-10 seconds)
- Include silence at beginning/end for timing analysis
- Use consistent sample rates (44.1kHz or 48kHz)

### 2. Data Management
- Version control test patches and reference files
- Use Git LFS for large audio files
- Document test patch parameters and expected behaviors
- Maintain separate branches for different SC versions

### 3. Result Analysis
- Focus on perceptually relevant metrics
- Use frequency-domain analysis for tonal content
- Compare channel energy distribution for spatial tests
- Account for floating-point precision differences

This setup provides a robust foundation for validating max2sc conversions through direct Max MSP / SuperCollider comparison.