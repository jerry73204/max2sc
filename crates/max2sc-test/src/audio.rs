//! Audio analysis and validation for testing

use crate::error::{Result, TestError};
use crate::runner::SCTestRunner;
use hound::{SampleFormat, WavReader};
use rustfft::{num_complex::Complex, FftPlanner};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, info};

/// Audio test configuration
#[derive(Debug)]
pub struct AudioTest {
    /// SuperCollider code to render
    pub code: String,
    /// Reference audio for comparison
    pub reference: AudioReference,
    /// Audio rendering settings
    pub settings: AudioSettings,
    /// Analysis parameters
    pub analysis: AudioAnalysis,
}

/// Reference audio for comparison
#[derive(Debug, Clone)]
pub struct AudioReference {
    /// Path to reference audio file
    pub file_path: PathBuf,
    /// Tolerance settings for comparison
    pub tolerance: AudioTolerance,
}

/// Audio rendering settings
#[derive(Debug, Clone)]
pub struct AudioSettings {
    /// Sample rate
    pub sample_rate: u32,
    /// Duration to render
    pub duration: Duration,
    /// Number of channels
    pub channels: u32,
    /// Bit depth
    pub bit_depth: u32,
}

/// Audio tolerance parameters
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioTolerance {
    /// Maximum RMS difference (0.0 - 1.0)
    pub rms_tolerance: f64,
    /// Spectral similarity threshold (0.0 - 1.0)
    pub spectral_similarity: f64,
    /// Phase coherence threshold (0.0 - 1.0)
    pub phase_coherence: f64,
    /// Peak amplitude tolerance
    pub peak_tolerance: f64,
    /// Frequency range for analysis (Hz)
    pub frequency_range: (f64, f64),
}

/// Audio analysis results
#[derive(Debug)]
pub struct AudioAnalysis {
    /// Whether the audio passed all tests
    pub passed: bool,
    /// RMS difference between test and reference
    pub rms_difference: f64,
    /// Spectral similarity (0.0 - 1.0)
    pub spectral_similarity: f64,
    /// Phase coherence (0.0 - 1.0)
    pub phase_coherence: f64,
    /// Peak amplitude difference
    pub peak_difference: f64,
    /// Frequency domain analysis
    pub frequency_analysis: FrequencyAnalysis,
    /// Time domain analysis
    pub time_analysis: TimeAnalysis,
    /// Detailed comparison metrics
    pub metrics: AudioMetrics,
}

/// Frequency domain analysis
#[derive(Debug)]
pub struct FrequencyAnalysis {
    /// Dominant frequencies in test audio
    pub test_peaks: Vec<FrequencyPeak>,
    /// Dominant frequencies in reference audio
    pub reference_peaks: Vec<FrequencyPeak>,
    /// Overall spectral centroid difference
    pub centroid_difference: f64,
    /// Spectral rolloff difference
    pub rolloff_difference: f64,
}

/// Time domain analysis
#[derive(Debug)]
pub struct TimeAnalysis {
    /// Zero crossing rate comparison
    pub zcr_difference: f64,
    /// Envelope correlation
    pub envelope_correlation: f64,
    /// Onset detection comparison
    pub onset_difference: f64,
}

/// Frequency peak information
#[derive(Debug)]
pub struct FrequencyPeak {
    /// Frequency in Hz
    pub frequency: f64,
    /// Magnitude (dB)
    pub magnitude: f64,
    /// Phase (radians)
    pub phase: f64,
}

/// Comprehensive audio metrics
#[derive(Debug)]
pub struct AudioMetrics {
    /// Signal-to-noise ratio
    pub snr_db: f64,
    /// Total harmonic distortion
    pub thd_percent: f64,
    /// Dynamic range
    pub dynamic_range_db: f64,
    /// Stereo width (for multi-channel)
    pub stereo_width: Option<f64>,
    /// Spatial accuracy (for spatial audio)
    pub spatial_accuracy: Option<f64>,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            duration: Duration::from_secs(5),
            channels: 2,
            bit_depth: 24,
        }
    }
}

impl Default for AudioTolerance {
    fn default() -> Self {
        Self {
            rms_tolerance: 0.05,              // 5% RMS difference
            spectral_similarity: 0.95,        // 95% spectral similarity
            phase_coherence: 0.90,            // 90% phase coherence
            peak_tolerance: 0.02,             // 2% peak difference
            frequency_range: (20.0, 20000.0), // Full audio range
        }
    }
}

impl AudioTest {
    /// Create a new audio test
    pub fn new(code: impl Into<String>, reference_path: impl AsRef<Path>) -> Self {
        Self {
            code: code.into(),
            reference: AudioReference {
                file_path: reference_path.as_ref().to_path_buf(),
                tolerance: AudioTolerance::default(),
            },
            settings: AudioSettings::default(),
            analysis: AudioAnalysis::default(),
        }
    }

    /// Set audio rendering settings
    pub fn with_settings(mut self, settings: AudioSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Set tolerance parameters
    pub fn with_tolerance(mut self, tolerance: AudioTolerance) -> Self {
        self.reference.tolerance = tolerance;
        self
    }

    /// Run the audio test
    pub async fn run(&self, runner: &SCTestRunner) -> Result<AudioAnalysis> {
        info!("Running audio test");

        // Generate audio rendering script
        let render_script = self.generate_render_script();
        let script_path = runner
            .create_temp_file("audio_test.scd", &render_script)
            .await?;
        let output_path = runner.temp_dir().join("test_output.wav");

        // Render audio
        debug!("Rendering test audio");
        let (stdout, stderr) = runner
            .execute_sclang(vec![
                script_path.to_string_lossy().to_string(),
                output_path.to_string_lossy().to_string(),
            ])
            .await?;

        // Check if rendering was successful
        if !output_path.exists() {
            return Err(TestError::AudioFile(format!(
                "Audio rendering failed. Stdout: {stdout}, Stderr: {stderr}"
            )));
        }

        // Load test and reference audio
        let test_audio = self.load_audio(&output_path)?;
        let reference_audio = self.load_audio(&self.reference.file_path)?;

        // Perform audio analysis
        self.analyze_audio(&test_audio, &reference_audio).await
    }

    /// Generate SuperCollider script for audio rendering
    fn generate_render_script(&self) -> String {
        format!(
            r#"
var outputPath = thisProcess.argv[0] ? "output.wav";

Server.default.options.sampleRate = {};
Server.default.options.numOutputBusChannels = {};
Server.default.options.blockSize = 64;

Server.default.waitForBoot {{
    var duration = {};
    var synth, buffer;
    
    // Create the test audio
    synth = {{
        var sig = {};
        Out.ar(0, sig);
    }}.play;
    
    // Record to buffer
    buffer = Buffer.alloc(Server.default, Server.default.sampleRate * duration, {});
    
    // Start recording
    buffer.write(outputPath, "wav", "int24", 0, 0, true);
    
    // Stop after duration
    SystemClock.sched(duration, {{
        synth.free;
        buffer.close;
        buffer.free;
        "Audio rendering completed".postln;
        0.exit;
    }});
}};
"#,
            self.settings.sample_rate,
            self.settings.channels,
            self.settings.duration.as_secs_f64(),
            self.code,
            self.settings.channels,
        )
    }

    /// Load audio file into memory
    fn load_audio(&self, path: &Path) -> Result<Vec<Vec<f32>>> {
        let mut reader = WavReader::open(path)
            .map_err(|e| TestError::AudioFile(format!("Failed to open audio file: {e}")))?;

        let spec = reader.spec();
        let mut samples: Vec<Vec<f32>> = vec![Vec::new(); spec.channels as usize];

        match spec.sample_format {
            SampleFormat::Int => {
                let max_val = (1_i32 << (spec.bits_per_sample - 1)) as f32;
                for (i, sample) in reader.samples::<i32>().enumerate() {
                    let sample = sample
                        .map_err(|e| TestError::AudioFile(format!("Sample read error: {e}")))?;
                    let normalized = sample as f32 / max_val;
                    samples[i % (spec.channels as usize)].push(normalized);
                }
            }
            SampleFormat::Float => {
                for (i, sample) in reader.samples::<f32>().enumerate() {
                    let sample = sample
                        .map_err(|e| TestError::AudioFile(format!("Sample read error: {e}")))?;
                    samples[i % (spec.channels as usize)].push(sample);
                }
            }
        }

        Ok(samples)
    }

    /// Perform comprehensive audio analysis
    async fn analyze_audio(
        &self,
        test_audio: &[Vec<f32>],
        reference_audio: &[Vec<f32>],
    ) -> Result<AudioAnalysis> {
        if test_audio.len() != reference_audio.len() {
            return Err(TestError::AudioAnalysis(
                "Channel count mismatch between test and reference audio".to_string(),
            ));
        }

        let mut total_rms_diff = 0.0;
        let mut total_peak_diff = 0.0;
        let mut spectral_similarities = Vec::new();

        // Analyze each channel
        for (channel, (test_ch, ref_ch)) in
            test_audio.iter().zip(reference_audio.iter()).enumerate()
        {
            debug!("Analyzing channel {}", channel);

            // RMS analysis
            let rms_diff = self.calculate_rms_difference(test_ch, ref_ch);
            total_rms_diff += rms_diff;

            // Peak analysis
            let peak_diff = self.calculate_peak_difference(test_ch, ref_ch);
            total_peak_diff += peak_diff;

            // Spectral analysis
            let spectral_sim = self.calculate_spectral_similarity(test_ch, ref_ch)?;
            spectral_similarities.push(spectral_sim);
        }

        // Average results across channels
        let num_channels = test_audio.len() as f64;
        let avg_rms_diff = total_rms_diff / num_channels;
        let avg_peak_diff = total_peak_diff / num_channels;
        let avg_spectral_sim = spectral_similarities.iter().sum::<f64>() / num_channels;

        // Generate frequency analysis
        let frequency_analysis =
            self.analyze_frequency_domain(&test_audio[0], &reference_audio[0])?;

        // Generate time analysis
        let time_analysis = self.analyze_time_domain(&test_audio[0], &reference_audio[0]);

        // Calculate overall metrics
        let metrics = self.calculate_audio_metrics(&test_audio[0]);

        // Determine if test passed
        let tolerance = &self.reference.tolerance;
        let passed = avg_rms_diff <= tolerance.rms_tolerance
            && avg_spectral_sim >= tolerance.spectral_similarity
            && avg_peak_diff <= tolerance.peak_tolerance;

        Ok(AudioAnalysis {
            passed,
            rms_difference: avg_rms_diff,
            spectral_similarity: avg_spectral_sim,
            phase_coherence: 0.95, // Placeholder - would need phase analysis
            peak_difference: avg_peak_diff,
            frequency_analysis,
            time_analysis,
            metrics,
        })
    }

    /// Calculate RMS difference between two signals
    fn calculate_rms_difference(&self, test: &[f32], reference: &[f32]) -> f64 {
        let min_len = test.len().min(reference.len());
        let mut sum_sq_diff = 0.0;
        let mut sum_sq_ref = 0.0;

        for i in 0..min_len {
            let diff = test[i] - reference[i];
            sum_sq_diff += (diff * diff) as f64;
            sum_sq_ref += (reference[i] * reference[i]) as f64;
        }

        if sum_sq_ref == 0.0 {
            return if sum_sq_diff == 0.0 { 0.0 } else { 1.0 };
        }

        (sum_sq_diff / sum_sq_ref).sqrt()
    }

    /// Calculate peak amplitude difference
    fn calculate_peak_difference(&self, test: &[f32], reference: &[f32]) -> f64 {
        let test_peak = test.iter().map(|x| x.abs()).fold(0.0, f32::max);
        let ref_peak = reference.iter().map(|x| x.abs()).fold(0.0, f32::max);

        if ref_peak == 0.0 {
            return if test_peak == 0.0 { 0.0 } else { 1.0 };
        }

        ((test_peak - ref_peak) / ref_peak).abs() as f64
    }

    /// Calculate spectral similarity using FFT
    fn calculate_spectral_similarity(&self, test: &[f32], reference: &[f32]) -> Result<f64> {
        let fft_size = 1024.min(test.len().min(reference.len()));
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);

        // Prepare test signal
        let mut test_complex: Vec<Complex<f32>> = test[..fft_size]
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();
        fft.process(&mut test_complex);

        // Prepare reference signal
        let mut ref_complex: Vec<Complex<f32>> = reference[..fft_size]
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();
        fft.process(&mut ref_complex);

        // Calculate magnitude spectra
        let test_mag: Vec<f64> = test_complex.iter().map(|c| c.norm() as f64).collect();
        let ref_mag: Vec<f64> = ref_complex.iter().map(|c| c.norm() as f64).collect();

        // Calculate correlation coefficient
        let mean_test = test_mag.iter().sum::<f64>() / test_mag.len() as f64;
        let mean_ref = ref_mag.iter().sum::<f64>() / ref_mag.len() as f64;

        let mut numerator = 0.0;
        let mut test_var = 0.0;
        let mut ref_var = 0.0;

        for i in 0..fft_size {
            let test_diff = test_mag[i] - mean_test;
            let ref_diff = ref_mag[i] - mean_ref;

            numerator += test_diff * ref_diff;
            test_var += test_diff * test_diff;
            ref_var += ref_diff * ref_diff;
        }

        if test_var == 0.0 || ref_var == 0.0 {
            return Ok(if test_var == ref_var { 1.0 } else { 0.0 });
        }

        Ok(numerator / (test_var * ref_var).sqrt())
    }

    /// Analyze frequency domain characteristics
    fn analyze_frequency_domain(
        &self,
        _test: &[f32],
        _reference: &[f32],
    ) -> Result<FrequencyAnalysis> {
        // Simplified frequency analysis - would be expanded in full implementation
        Ok(FrequencyAnalysis {
            test_peaks: Vec::new(),
            reference_peaks: Vec::new(),
            centroid_difference: 0.0,
            rolloff_difference: 0.0,
        })
    }

    /// Analyze time domain characteristics
    fn analyze_time_domain(&self, _test: &[f32], _reference: &[f32]) -> TimeAnalysis {
        // Simplified time analysis - would be expanded in full implementation
        TimeAnalysis {
            zcr_difference: 0.0,
            envelope_correlation: 1.0,
            onset_difference: 0.0,
        }
    }

    /// Calculate comprehensive audio metrics
    fn calculate_audio_metrics(&self, _audio: &[f32]) -> AudioMetrics {
        // Simplified metrics calculation - would be expanded in full implementation
        AudioMetrics {
            snr_db: 60.0,
            thd_percent: 0.1,
            dynamic_range_db: 80.0,
            stereo_width: None,
            spatial_accuracy: None,
        }
    }
}

impl Default for AudioAnalysis {
    fn default() -> Self {
        Self {
            passed: false,
            rms_difference: 0.0,
            spectral_similarity: 0.0,
            phase_coherence: 0.0,
            peak_difference: 0.0,
            frequency_analysis: FrequencyAnalysis {
                test_peaks: Vec::new(),
                reference_peaks: Vec::new(),
                centroid_difference: 0.0,
                rolloff_difference: 0.0,
            },
            time_analysis: TimeAnalysis {
                zcr_difference: 0.0,
                envelope_correlation: 0.0,
                onset_difference: 0.0,
            },
            metrics: AudioMetrics {
                snr_db: 0.0,
                thd_percent: 0.0,
                dynamic_range_db: 0.0,
                stereo_width: None,
                spatial_accuracy: None,
            },
        }
    }
}

/// Audio file comparison utility
pub struct AudioComparison {
    /// Path to first audio file
    file1: PathBuf,
    /// Path to second audio file
    file2: PathBuf,
}

/// Result of audio comparison
#[derive(Debug, Clone)]
pub struct AudioComparisonResult {
    /// Overall similarity score (0.0 - 1.0)
    pub similarity: f32,
    /// RMS difference
    pub rms_difference: f64,
    /// Spectral similarity
    pub spectral_similarity: f64,
    /// Peak difference
    pub peak_difference: f64,
    /// Whether comparison passed
    pub passed: bool,
}

impl AudioComparison {
    /// Create new audio comparison
    pub fn new(file1: &Path, file2: &Path) -> Result<Self> {
        if !file1.exists() {
            return Err(TestError::AudioFile(format!("File not found: {}", file1.display())));
        }
        if !file2.exists() {
            return Err(TestError::AudioFile(format!("File not found: {}", file2.display())));
        }

        Ok(Self {
            file1: file1.to_path_buf(),
            file2: file2.to_path_buf(),
        })
    }

    /// Compare two audio files
    pub fn compare(&self, tolerance: AudioTolerance) -> Result<AudioComparisonResult> {
        let audio1 = self.load_audio_file(&self.file1)?;
        let audio2 = self.load_audio_file(&self.file2)?;

        if audio1.len() != audio2.len() {
            return Err(TestError::AudioAnalysis(
                "Audio files have different number of channels".to_string(),
            ));
        }

        // Calculate RMS difference
        let rms_diff = self.calculate_rms_difference(&audio1[0], &audio2[0]);
        
        // Calculate spectral similarity
        let spectral_sim = self.calculate_spectral_similarity(&audio1[0], &audio2[0])?;
        
        // Calculate peak difference
        let peak_diff = self.calculate_peak_difference(&audio1[0], &audio2[0]);

        // Overall similarity score
        let similarity = ((1.0 - rms_diff as f32) + spectral_sim as f32 + (1.0 - peak_diff as f32)) / 3.0;

        let passed = rms_diff <= tolerance.rms_tolerance
            && spectral_sim >= tolerance.spectral_similarity
            && peak_diff <= tolerance.peak_tolerance;

        Ok(AudioComparisonResult {
            similarity,
            rms_difference: rms_diff,
            spectral_similarity: spectral_sim,
            peak_difference: peak_diff,
            passed,
        })
    }

    /// Load audio file into memory
    fn load_audio_file(&self, path: &Path) -> Result<Vec<Vec<f32>>> {
        let mut reader = WavReader::open(path)
            .map_err(|e| TestError::AudioFile(format!("Failed to open {}: {}", path.display(), e)))?;

        let spec = reader.spec();
        let mut samples: Vec<Vec<f32>> = vec![Vec::new(); spec.channels as usize];

        match spec.sample_format {
            SampleFormat::Int => {
                let max_val = (1_i32 << (spec.bits_per_sample - 1)) as f32;
                for (i, sample) in reader.samples::<i32>().enumerate() {
                    let sample = sample
                        .map_err(|e| TestError::AudioFile(format!("Sample read error: {e}")))?;
                    let normalized = sample as f32 / max_val;
                    samples[i % (spec.channels as usize)].push(normalized);
                }
            }
            SampleFormat::Float => {
                for (i, sample) in reader.samples::<f32>().enumerate() {
                    let sample = sample
                        .map_err(|e| TestError::AudioFile(format!("Sample read error: {e}")))?;
                    samples[i % (spec.channels as usize)].push(sample);
                }
            }
        }

        Ok(samples)
    }

    /// Calculate RMS difference between two signals
    fn calculate_rms_difference(&self, signal1: &[f32], signal2: &[f32]) -> f64 {
        let min_len = signal1.len().min(signal2.len());
        if min_len == 0 {
            return 1.0;
        }

        let mut sum_sq = 0.0;
        for i in 0..min_len {
            let diff = signal1[i] - signal2[i];
            sum_sq += (diff * diff) as f64;
        }

        (sum_sq / min_len as f64).sqrt()
    }

    /// Calculate spectral similarity
    fn calculate_spectral_similarity(&self, signal1: &[f32], signal2: &[f32]) -> Result<f64> {
        // Simplified spectral similarity - could be improved with proper FFT
        let min_len = signal1.len().min(signal2.len());
        if min_len < 256 {
            return Ok(0.5); // Default similarity for short signals
        }

        // Use sliding window correlation as a proxy for spectral similarity
        let window_size = 256;
        let mut correlations = Vec::new();

        for start in (0..min_len - window_size).step_by(window_size / 2) {
            let end = start + window_size;
            let corr = self.calculate_correlation(&signal1[start..end], &signal2[start..end]);
            correlations.push(corr);
        }

        if correlations.is_empty() {
            Ok(0.5)
        } else {
            Ok(correlations.iter().sum::<f64>() / correlations.len() as f64)
        }
    }

    /// Calculate correlation between two signals
    fn calculate_correlation(&self, signal1: &[f32], signal2: &[f32]) -> f64 {
        let n = signal1.len().min(signal2.len());
        if n == 0 {
            return 0.0;
        }

        let mean1: f64 = signal1.iter().take(n).map(|&x| x as f64).sum::<f64>() / n as f64;
        let mean2: f64 = signal2.iter().take(n).map(|&x| x as f64).sum::<f64>() / n as f64;

        let mut numerator = 0.0;
        let mut var1 = 0.0;
        let mut var2 = 0.0;

        for i in 0..n {
            let diff1 = signal1[i] as f64 - mean1;
            let diff2 = signal2[i] as f64 - mean2;

            numerator += diff1 * diff2;
            var1 += diff1 * diff1;
            var2 += diff2 * diff2;
        }

        if var1 == 0.0 || var2 == 0.0 {
            if var1 == var2 { 1.0 } else { 0.0 }
        } else {
            numerator / (var1 * var2).sqrt()
        }
    }

    /// Calculate peak difference
    fn calculate_peak_difference(&self, signal1: &[f32], signal2: &[f32]) -> f64 {
        let peak1 = signal1.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        let peak2 = signal2.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);

        ((peak1 - peak2).abs() / peak1.max(peak2).max(1e-10)) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_settings_default() {
        let settings = AudioSettings::default();
        assert_eq!(settings.sample_rate, 48000);
        assert_eq!(settings.channels, 2);
        assert_eq!(settings.duration, Duration::from_secs(5));
    }

    #[test]
    fn test_audio_tolerance_default() {
        let tolerance = AudioTolerance::default();
        assert_eq!(tolerance.rms_tolerance, 0.05);
        assert_eq!(tolerance.spectral_similarity, 0.95);
        assert_eq!(tolerance.frequency_range, (20.0, 20000.0));
    }

    #[test]
    fn test_rms_calculation() {
        let test = AudioTest::new("SinOsc.ar(440)", "reference.wav");

        let signal1 = vec![1.0, 0.0, -1.0, 0.0];
        let signal2 = vec![1.0, 0.0, -1.0, 0.0];
        let rms_diff = test.calculate_rms_difference(&signal1, &signal2);
        assert!((rms_diff - 0.0).abs() < 1e-10);

        let signal3 = vec![0.5, 0.0, -0.5, 0.0];
        let rms_diff = test.calculate_rms_difference(&signal1, &signal3);
        assert!(rms_diff > 0.0);
    }
}
