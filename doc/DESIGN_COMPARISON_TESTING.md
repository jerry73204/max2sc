# Max MSP to SuperCollider Output Comparison Strategy

## Overview

This document outlines strategies for comparing audio and control outputs between Max MSP (running on WINE) and SuperCollider (native Linux) to validate the accuracy of max2sc conversions.

## 1. Audio Output Comparison

### 1.1 File-Based Comparison

The most reliable method for cross-platform comparison is to render audio to files and compare them.

#### Max MSP Side (WINE)
```max
// Record audio output to file
[sfrecord~ 2]  // stereo recording
|
[open /tmp/max_output.wav]
|
[toggle] // start/stop recording
```

#### SuperCollider Side
```supercollider
// Record to file using Score.recordNRT or real-time recording
s.record("/tmp/sc_output.wav");
// ... your synth here ...
s.stopRecording;

// Or use NRT rendering for deterministic output
Score.recordNRT(
    [
        [0.0, [\s_new, \testSynth, 1000, 0, 0]],
        [10.0, [\c_set, 0, 0]]
    ],
    "/tmp/sc_output.wav",
    sampleRate: 44100,
    options: ServerOptions.new
        .numOutputBusChannels_(2)
);
```

### 1.2 Audio Analysis Tools

#### Using the max2sc-test Framework
```rust
use max2sc_test::audio::{AudioComparison, AudioTolerance};

let comparison = AudioComparison::new(
    "/tmp/max_output.wav",
    "/tmp/sc_output.wav"
)?;

let tolerance = AudioTolerance {
    rms_tolerance: 0.01,        // 1% RMS difference
    spectral_similarity: 0.95,   // 95% spectral similarity
    phase_coherence: 0.9,        // 90% phase coherence
    peak_tolerance: 0.02,        // 2% peak difference
    frequency_range: (20.0, 20000.0),
};

let result = comparison.compare(tolerance)?;
```

#### Command-Line Tools
```bash
# Using sox for basic comparison
sox /tmp/max_output.wav -n stat 2>&1
sox /tmp/sc_output.wav -n stat 2>&1

# Using ffmpeg for spectral analysis
ffmpeg -i /tmp/max_output.wav -lavfi showspectrumpic=s=1024x512 max_spectrum.png
ffmpeg -i /tmp/sc_output.wav -lavfi showspectrumpic=s=1024x512 sc_spectrum.png

# Using aubio tools for detailed analysis
aubioonset /tmp/max_output.wav
aubioonset /tmp/sc_output.wav
```

## 2. OSC Communication Comparison

### 2.1 OSC Message Logging

#### Max MSP OSC Logger
```max
// Create OSC logger patch
[udpreceive 9000]
|
[route /spat /source /reverb]
|
[print osc_received]
|
[capture] // save to text file
```

#### SuperCollider OSC Logger
```supercollider
// Log all OSC messages
OSCdef(\logger, { |msg, time, addr, recvPort|
    var file = File("/tmp/sc_osc_log.txt", "a");
    file.write(format("% % %\n", time, addr, msg));
    file.close;
}, nil, nil, 9000);
```

### 2.2 OSC Comparison Script
```python
#!/usr/bin/env python3
# compare_osc_logs.py

import re
from difflib import unified_diff

def parse_osc_log(filename):
    """Parse OSC log file into normalized format"""
    messages = []
    with open(filename) as f:
        for line in f:
            # Extract timestamp, address, and values
            match = re.match(r'(\d+\.?\d*)\s+(\S+)\s+(.+)', line)
            if match:
                time, addr, values = match.groups()
                messages.append((float(time), addr, values))
    return messages

def compare_osc_logs(max_log, sc_log, time_tolerance=0.01):
    """Compare two OSC log files"""
    max_msgs = parse_osc_log(max_log)
    sc_msgs = parse_osc_log(sc_log)
    
    # Compare messages within time tolerance
    differences = []
    for max_msg in max_msgs:
        found = False
        for sc_msg in sc_msgs:
            if (abs(max_msg[0] - sc_msg[0]) < time_tolerance and
                max_msg[1] == sc_msg[1]):
                if max_msg[2] != sc_msg[2]:
                    differences.append({
                        'time': max_msg[0],
                        'address': max_msg[1],
                        'max_values': max_msg[2],
                        'sc_values': sc_msg[2]
                    })
                found = True
                break
        if not found:
            differences.append({
                'time': max_msg[0],
                'address': max_msg[1],
                'max_values': max_msg[2],
                'sc_values': 'MISSING'
            })
    
    return differences
```

## 3. Automated Comparison Pipeline

### 3.1 Test Harness Architecture

```rust
// crates/max2sc-test/src/comparison.rs

pub struct Max2SCComparison {
    max_runner: MaxRunner,      // Controls Max via WINE
    sc_runner: SCTestRunner,    // Native SC runner
    comparison: ComparisonConfig,
}

impl Max2SCComparison {
    pub async fn run_comparison_test(
        &self,
        max_patch: &Path,
        sc_code: &str,
        test_duration: Duration,
    ) -> Result<ComparisonResult> {
        // 1. Start Max patch in WINE
        let max_output = self.max_runner.render_to_file(
            max_patch,
            test_duration
        ).await?;
        
        // 2. Run SC code
        let sc_output = self.sc_runner.render_to_file(
            sc_code,
            test_duration
        ).await?;
        
        // 3. Compare outputs
        let audio_comparison = AudioComparison::new(
            &max_output,
            &sc_output
        )?;
        
        Ok(ComparisonResult {
            audio_similarity: audio_comparison.compare(self.comparison.tolerance)?,
            timing_accuracy: self.compare_timing(&max_output, &sc_output)?,
            spectral_match: self.compare_spectra(&max_output, &sc_output)?,
        })
    }
}
```

### 3.2 WINE Integration for Max

```rust
// crates/max2sc-test/src/max_runner.rs

pub struct MaxRunner {
    wine_prefix: PathBuf,
    max_executable: PathBuf,
    temp_dir: TempDir,
}

impl MaxRunner {
    pub async fn render_to_file(
        &self,
        patch: &Path,
        duration: Duration,
    ) -> Result<PathBuf> {
        // Create control patch that loads target and records
        let control_patch = self.create_recording_patch(patch, duration)?;
        
        // Run Max in WINE
        let output = Command::new("wine")
            .env("WINEPREFIX", &self.wine_prefix)
            .env("WINEDEBUG", "-all")  // Reduce noise
            .arg(&self.max_executable)
            .arg("-nogui")  // Headless mode
            .arg("-nosplash")
            .arg(control_patch)
            .output()
            .await?;
        
        // Return path to recorded audio
        Ok(self.temp_dir.path().join("max_output.wav"))
    }
}
```

## 4. Spatial Audio Comparison

### 4.1 Multichannel Recording Setup

```supercollider
// Record 8-channel spatial output
s.options.numOutputBusChannels = 8;
s.options.numInputBusChannels = 8;

// Record all channels
s.record("/tmp/sc_8ch.wav", numChannels: 8);

// Compare channel levels
~analyzeChannels = { |path|
    var sf = SoundFile.openRead(path);
    var channelRMS = Array.fill(sf.numChannels, 0);
    var frame = FloatArray.newClear(sf.numChannels);
    
    while { sf.readData(frame) } {
        frame.do { |sample, i|
            channelRMS[i] = channelRMS[i] + sample.squared;
        };
    };
    
    channelRMS = channelRMS.sqrt / sf.numFrames.sqrt;
    sf.close;
    channelRMS
};
```

### 4.2 Spatial Accuracy Metrics

```rust
pub struct SpatialComparison {
    /// Channel energy distribution comparison
    pub channel_distribution: f32,
    
    /// Inter-channel correlation
    pub spatial_correlation: f32,
    
    /// Localization accuracy (ITD/ILD)
    pub localization_accuracy: f32,
    
    /// Distance cue preservation
    pub distance_accuracy: f32,
}

impl SpatialComparison {
    pub fn compare_vbap_panning(
        max_audio: &AudioFile,
        sc_audio: &AudioFile,
        speaker_config: &SpeakerConfiguration,
    ) -> Result<Self> {
        // Compare energy distribution across speakers
        let max_distribution = Self::calculate_channel_energy(max_audio)?;
        let sc_distribution = Self::calculate_channel_energy(sc_audio)?;
        
        // Calculate correlation between channel pairs
        let correlation = Self::calculate_spatial_correlation(
            &max_distribution,
            &sc_distribution
        )?;
        
        Ok(Self {
            channel_distribution: correlation,
            // ... other metrics
        })
    }
}
```

## 5. Headless Testing Setup

### 5.1 Docker Configuration

```dockerfile
# Dockerfile for max2sc comparison testing
FROM ubuntu:22.04

# Install WINE and dependencies
RUN dpkg --add-architecture i386 && \
    apt-get update && \
    apt-get install -y \
        wine wine32 wine64 \
        supercollider-headless \
        sox aubio-tools \
        python3 python3-pip

# Install Max MSP in WINE (requires license)
COPY max_installer.exe /tmp/
RUN wine /tmp/max_installer.exe /SILENT

# Install comparison tools
COPY requirements.txt /tmp/
RUN pip3 install -r /tmp/requirements.txt

# Set up headless audio
RUN apt-get install -y jackd2 pulseaudio
COPY jackd_headless.conf /etc/

WORKDIR /workspace
```

### 5.2 CI/CD Pipeline

```yaml
# .github/workflows/comparison-tests.yml
name: Max/SC Comparison Tests

on: [push, pull_request]

jobs:
  comparison-tests:
    runs-on: ubuntu-latest
    container:
      image: max2sc/comparison-test:latest
      
    steps:
      - uses: actions/checkout@v3
      
      - name: Start virtual audio
        run: |
          jackd -d dummy -r 44100 -p 512 &
          sleep 2
          
      - name: Run comparison tests
        run: |
          cargo test --features comparison-tests
          
      - name: Upload comparison reports
        uses: actions/upload-artifact@v3
        with:
          name: comparison-results
          path: target/comparison_reports/
```

## 6. Test Data Management

### 6.1 Reference Patches

Create a set of reference Max patches for comparison:

```
tests/comparison/
├── basic/
│   ├── sine_oscillator.maxpat
│   ├── multichannel_test.maxpat
│   └── osc_routing.maxpat
├── spatial/
│   ├── vbap_8ch.maxpat
│   ├── hoa_3rd_order.maxpat
│   └── wfs_linear_array.maxpat
└── expected/
    ├── sine_oscillator.wav
    ├── vbap_8ch_45deg.wav
    └── comparison_tolerances.json
```

### 6.2 Tolerance Configuration

```json
{
  "default": {
    "audio": {
      "rms_tolerance": 0.01,
      "peak_tolerance": 0.02,
      "spectral_similarity": 0.95
    },
    "timing": {
      "onset_tolerance_ms": 5,
      "duration_tolerance_ms": 10
    }
  },
  "spatial": {
    "audio": {
      "channel_balance_tolerance": 0.05,
      "correlation_threshold": 0.9
    }
  },
  "effects": {
    "audio": {
      "rms_tolerance": 0.05,  // Higher tolerance for effects
      "spectral_similarity": 0.85
    }
  }
}
```

## 7. Reporting and Visualization

### 7.1 Comparison Report Generator

```rust
pub fn generate_comparison_report(
    results: &[ComparisonResult],
) -> Result<String> {
    let mut report = String::new();
    
    writeln!(report, "# Max2SC Comparison Report")?;
    writeln!(report, "\n## Summary")?;
    writeln!(report, "- Total tests: {}", results.len())?;
    writeln!(report, "- Passed: {}", results.iter().filter(|r| r.passed).count())?;
    
    writeln!(report, "\n## Detailed Results")?;
    for result in results {
        writeln!(report, "\n### {}", result.test_name)?;
        writeln!(report, "- Audio similarity: {:.2}%", result.audio_similarity * 100.0)?;
        writeln!(report, "- Timing accuracy: {:.2}%", result.timing_accuracy * 100.0)?;
        
        if !result.passed {
            writeln!(report, "\n**Differences:**")?;
            for diff in &result.differences {
                writeln!(report, "- {}", diff)?;
            }
        }
    }
    
    Ok(report)
}
```

## 8. Practical Considerations

### 8.1 WINE Configuration

```bash
# Set up WINE for Max MSP
export WINEPREFIX=$HOME/.wine_max
export WINEARCH=win64

# Configure audio
winecfg  # Set audio to ALSA or PulseAudio

# Install dependencies
winetricks corefonts vcrun2019 dotnet48
```

### 8.2 Latency Compensation

When comparing real-time outputs, account for system latency:

```supercollider
// Measure and compensate for latency
s.latency = 0.05;  // 50ms latency compensation

// Use timestamps for accurate comparison
~timestamp = { |label|
    postf("% at %\n", label, Main.elapsedTime);
};
```

## Conclusion

This comparison strategy provides multiple approaches for validating max2sc conversions:
1. File-based audio comparison for accuracy
2. OSC message logging for control validation  
3. Automated testing with WINE integration
4. Spatial audio validation metrics
5. Headless CI/CD compatibility

The combination of these methods ensures comprehensive validation of the conversion process across different platforms and environments.