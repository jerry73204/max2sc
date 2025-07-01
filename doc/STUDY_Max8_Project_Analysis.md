# Max8 Project Analysis Notes

## Project Overview
The `max8_source/` directory contains a comprehensive Max MSP 8 project focused on spatial audio, particularly Wave Field Synthesis (WFS) and multi-speaker arrays.

## Key Components

### 1. Main Patch: AUO_2024_Max8_copy.maxpat
- **Version**: Max 8.6.5 (x64)
- **Key Features**:
  - 82-channel multichannel processing using `mc.unpack~ 82`
  - Multiple output routing (up to 128 outputs)
  - `mc.live.gain~` for multichannel gain control
  - Extensive use of `adstatus output` for audio device routing

### 2. Speaker Configuration Files
Multiple speaker array configurations are present:
- `AUO_32speakers_45mm.txt` - 32-speaker WFS array
- `AUO_48speakers_45mm.txt` - 48-speaker array
- `AUO_64speakers_45mm.txt` - 64-speaker array
- `pilotStudy_16speakers_45mm.txt` - 16-speaker pilot study
- Various dated speaker configuration files (0311.txt, 0514spat.txt, etc.)

#### Configuration Format (OSC-based):
- Master bus configuration with up to 32 I/O
- WFS Bus configurations (Bus 1: 16 channels, Bus 2: 32 channels)
- Speaker positioning data (azimuth, elevation, distance)
- Delay and gain correction values per speaker
- Reverb settings with cluster delays
- Equalizer settings (8-band parametric EQ)
- Dynamics processing (compressor/expander)

### 3. Audio Content
- **Musical samples**: Various orchestral instruments (Violin, Piccolo, Oboe, Clarinet, etc.)
- **Sound effects**: Game sounds, ambient recordings, fireworks
- **Test signals**: Various WAV and MP3 files for spatial testing
- **Speech samples**: Male and female speech recordings

### 4. SPAT5 Integration
The project includes the complete SPAT5 package:
- **Core objects**: `spat5.panoramix~`, `spat5.osc.route`
- **Tutorials**: Comprehensive tutorial patches for:
  - Basic spatial audio
  - VBAP (Vector Base Amplitude Panning)
  - HOA (Higher Order Ambisonics)
  - Binaural processing
  - WFS (Wave Field Synthesis)
  - IKO speaker arrays
- **ADM support**: Audio Definition Model renderer and monitoring

### 5. Spatial Audio Features

#### Spatialization Methods:
1. **WFS (Wave Field Synthesis)**
   - Multiple WFS bus configurations
   - Speaker array delay/gain compensation
   - Pre-filtering options
   - LSA (Line Source Array) mode support

2. **Ambisonics**
   - HOA encoding/decoding
   - Speaker array files: `ambiSpeakerArray.txt`, `ambiSpeakerCOLL.txt`

3. **VBAP**
   - Vector-based amplitude panning
   - 3D positioning (azimuth, elevation, distance)

4. **Reverb Processing**
   - Multiple reverb buses
   - Early reflection control
   - Cluster delay configurations
   - Frequency-dependent reverberation times

#### Control Features:
- OSC control protocol
- Snapshot/preset management
- Real-time parameter interpolation
- Doppler effect control
- Air absorption modeling
- Distance-based gain control

### 6. Technical Specifications

#### Audio Processing:
- Sample rates: Appears to support standard rates
- Channel counts: Up to 128 simultaneous channels
- Latency compensation: Zero-latency normalization modes
- Interpolation: Lagrange3 interpolation for smooth transitions

#### Signal Flow:
1. Input sources (mono/stereo tracks)
2. Spatial processing (position, distance, etc.)
3. Bus routing (multiple WFS/reverb buses)
4. Speaker-specific processing (delays, gains, EQ)
5. Master output section

### 7. JavaScript Components
- `nfc.js` - Likely Near Field Compensation
- `Untitled.js` - Additional scripting support

## Conversion Challenges

1. **Multichannel Architecture**: Max's mc.* objects need mapping to SC's array expansion
2. **SPAT5 Objects**: Complex spatial algorithms requiring SC equivalents
3. **OSC Configuration**: Speaker array data needs parsing and conversion
4. **Real-time Control**: Parameter interpolation and ramping
5. **GUI Elements**: Visual feedback not directly translatable

## Priority Objects for Conversion

1. **Core Audio**:
   - `mc.unpack~` → SC array routing
   - `mc.live.gain~` → SC gain control arrays
   - `adstatus output` → SC audio output configuration

2. **Spatial Processing**:
   - `spat5.panoramix~` → Custom SC spatialization synths
   - WFS processing → SC WFS implementations
   - HOA encoding/decoding → SC HOA UGens

3. **Control Infrastructure**:
   - OSC routing → SC OSC responders
   - Preset management → SC pattern system
   - Parameter interpolation → SC envelopes/lag

## Next Steps

1. Parse speaker configuration files to extract array geometry
2. Map SPAT5 spatial algorithms to SC equivalents
3. Design multichannel routing architecture in SC
4. Implement OSC control protocol compatibility
5. Create test cases for each spatial processing method