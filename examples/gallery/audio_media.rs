//! Zero-dependency audio engine, PCM WAV parser, procedural synthesizer, and real-time VU meter calculator.

/// Digital audio track containing normalized samples, computed waveform peaks, and duration.
#[derive(Clone)]
pub struct AudioTrack {
    pub name: String,
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub samples_mono: Vec<f32>,
    pub waveform_peaks: Vec<f32>,
}

impl Default for AudioTrack {
    fn default() -> Self {
        Self::procedural_studio_track("AppKit_Master_Mix.wav", 30.0)
    }
}

impl AudioTrack {
    /// Generates an authentic multi-layer procedural studio audio track with melody and rhythm.
    pub fn procedural_studio_track(name: &str, duration_secs: f64) -> Self {
        let sample_rate = 22050u32;
        let synth_duration = duration_secs.clamp(1.0, 120.0);
        let total_samples = (synth_duration * sample_rate as f64).round() as usize;
        let mut samples = Vec::with_capacity(total_samples);

        for i in 0..total_samples {
            let t = i as f64 / sample_rate as f64;
            // Beat kick & bass (4-on-the-floor)
            let beat = (t * 2.0).fract();
            let kick = (-beat * 24.0).exp() * (beat * 140.0 * std::f64::consts::TAU).sin() * 0.45;
            // Harmonic synth pad
            let pad = (t * 220.0 * std::f64::consts::TAU).sin() * 0.18
                + (t * 330.0 * std::f64::consts::TAU).sin() * 0.12
                + (t * 440.0 * std::f64::consts::TAU).sin() * 0.08;
            // High-hat shimmer
            let hat_phase = (t * 4.0).fract();
            let hat = if hat_phase < 0.08 {
                let pseudo_noise =
                    ((i * 1103515245 + 12345) & 0x7FFFFFFF) as f64 / 2147483648.0 - 0.5;
                pseudo_noise * (-hat_phase * 60.0).exp() * 0.22
            } else {
                0.0
            };
            let mixed = (kick + pad + hat).clamp(-1.0, 1.0) as f32;
            samples.push(mixed);
        }

        let peaks = compute_waveform_peaks(&samples, 96);
        Self {
            name: name.to_string(),
            duration_secs: synth_duration,
            sample_rate,
            samples_mono: samples,
            waveform_peaks: peaks,
        }
    }

    /// Parses a standard RIFF/WAVE PCM audio file buffer from disk or drag-and-drop.
    pub fn from_wav_bytes(name: &str, bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
            return Err("Invalid WAV container header".to_string());
        }

        let mut offset = 12usize;
        let mut fmt_found = false;
        let (mut channels, mut sample_rate, mut bits_per_sample) = (0u16, 0u32, 0u16);
        let mut data_slice: Option<&[u8]> = None;

        while offset + 8 <= bytes.len() {
            let chunk_id = &bytes[offset..offset + 4];
            let chunk_len = u32::from_le_bytes(
                bytes
                    .get(offset + 4..offset + 8)
                    .ok_or("Corrupt chunk header")?
                    .try_into()
                    .unwrap(),
            ) as usize;
            offset += 8;

            if chunk_id == b"fmt " {
                if chunk_len < 16 || offset + 16 > bytes.len() {
                    return Err("Truncated fmt chunk in WAV file".to_string());
                }
                let audio_format =
                    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap());
                if audio_format != 1 && audio_format != 3 {
                    return Err(format!(
                        "Unsupported WAV compression format: {audio_format}"
                    ));
                }
                channels = u16::from_le_bytes(bytes[offset + 2..offset + 4].try_into().unwrap());
                sample_rate = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
                bits_per_sample =
                    u16::from_le_bytes(bytes[offset + 14..offset + 16].try_into().unwrap());
                if channels == 0 || sample_rate == 0 {
                    return Err("Invalid zero channels or sample rate".to_string());
                }
                if !matches!(bits_per_sample, 8 | 16 | 24 | 32) {
                    return Err(format!("Unsupported bit depth: {bits_per_sample}"));
                }
                fmt_found = true;
            } else if chunk_id == b"data" {
                let actual_len = chunk_len.min(bytes.len().saturating_sub(offset));
                data_slice = Some(&bytes[offset..offset + actual_len]);
                break;
            }
            // RIFF chunks are 2-byte aligned
            offset = (offset.saturating_add(chunk_len) + 1) & !1;
        }

        if !fmt_found || data_slice.is_none() {
            return Err("Missing fmt or data chunk in WAV file".to_string());
        }
        let data = data_slice.unwrap();
        let bytes_per_sample = (bits_per_sample / 8) as usize;
        let frame_size = bytes_per_sample * channels as usize;
        if frame_size == 0 {
            return Err("Invalid zero frame size".to_string());
        }
        let num_frames = data.len() / frame_size;

        let mut samples_mono = Vec::with_capacity(num_frames);
        for frame_idx in 0..num_frames {
            let frame_offset = frame_idx * frame_size;
            let mut sum_frame = 0.0f32;
            for ch in 0..channels as usize {
                let s_off = frame_offset + ch * bytes_per_sample;
                if s_off + bytes_per_sample > data.len() {
                    break;
                }
                let s_val = match bits_per_sample {
                    8 => {
                        let raw = data[s_off];
                        (raw as f32 - 128.0) / 128.0
                    }
                    16 => {
                        let raw = i16::from_le_bytes(data[s_off..s_off + 2].try_into().unwrap());
                        raw as f32 / 32768.0
                    }
                    24 => {
                        let b = [data[s_off], data[s_off + 1], data[s_off + 2], 0];
                        let raw = (i32::from_le_bytes(b) << 8) >> 8;
                        raw as f32 / 8388608.0
                    }
                    32 => {
                        let raw = f32::from_le_bytes(data[s_off..s_off + 4].try_into().unwrap());
                        raw.clamp(-1.0, 1.0)
                    }
                    _ => 0.0,
                };
                sum_frame += s_val;
            }
            samples_mono.push(sum_frame / (channels as f32));
        }

        let duration_secs = num_frames as f64 / sample_rate as f64;
        let waveform_peaks = compute_waveform_peaks(&samples_mono, 96);
        Ok(Self {
            name: name.to_string(),
            duration_secs,
            sample_rate,
            samples_mono,
            waveform_peaks,
        })
    }

    /// Computes true RMS and peak audio meters for stereo L/R at the specified playhead.
    pub fn compute_stereo_levels_at(
        &self,
        time_secs: f64,
        gain: f32,
        pan: f32,
    ) -> (f32, f32, f32, f32) {
        if self.samples_mono.is_empty() || self.duration_secs <= 0.0 {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let norm_t = (time_secs / self.duration_secs.max(1.0)).clamp(0.0, 1.0);
        let center_sample =
            (norm_t * (self.samples_mono.len().saturating_sub(1) as f64)).round() as usize;
        let window_size = (self.sample_rate as usize / 25).max(128); // 40ms metering window
        let start = center_sample.saturating_sub(window_size / 2);
        let end = (start + window_size).min(self.samples_mono.len());
        let slice = &self.samples_mono[start..end];

        if slice.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let mut sum_sq = 0.0f32;
        let mut peak_val = 0.0f32;
        for &s in slice {
            let abs_s = s.abs();
            if abs_s > peak_val {
                peak_val = abs_s;
            }
            sum_sq += s * s;
        }
        let rms = (sum_sq / (slice.len() as f32)).sqrt();

        // Pan weighting (-1.0=Left, 0.0=Center, 1.0=Right)
        let (pan_l, pan_r) = calculate_pan_weights(pan);

        // Map to 0.0..=10.0 MacLevelIndicator scale (-48dBFS to +3dBFS)
        let to_vu = |amp: f32| -> f32 {
            let amp_scaled = (amp * gain.abs()).abs();
            if amp_scaled.is_nan() || amp_scaled < 1e-4 {
                0.0
            } else {
                let db = 20.0 * amp_scaled.log10();
                ((db + 48.0) / 48.0 * 9.5).clamp(0.0, 10.0)
            }
        };

        let level_l = to_vu(rms * pan_l);
        let level_r = to_vu(rms * pan_r);
        let peak_l = to_vu(peak_val * pan_l).max(level_l);
        let peak_r = to_vu(peak_val * pan_r).max(level_r);
        (level_l, level_r, peak_l, peak_r)
    }

    /// Encodes a portion of the track to standard 16-bit PCM WAV for native OS playback with gain and stereo pan.
    pub fn to_wav_bytes_mixed(
        &self,
        start_secs: f64,
        end_secs: f64,
        gain: f32,
        pan: f32,
    ) -> Vec<u8> {
        let (s_sec, e_sec) = if start_secs <= end_secs {
            (start_secs, end_secs)
        } else {
            (end_secs, start_secs)
        };
        let dur = self.duration_secs.max(1.0);
        let start_frame =
            (((s_sec / dur).clamp(0.0, 1.0)) * self.samples_mono.len() as f64).round() as usize;
        let end_frame =
            (((e_sec / dur).clamp(0.0, 1.0)) * self.samples_mono.len() as f64).round() as usize;
        let s_idx = start_frame.min(self.samples_mono.len());
        let e_idx = end_frame.min(self.samples_mono.len()).max(s_idx);
        let slice = &self.samples_mono[s_idx..e_idx];

        let num_samples = slice.len();
        let data_len = (num_samples * 2 * 2) as u32; // 16-bit stereo
        let file_len = 36 + data_len;

        let mut out = Vec::with_capacity(44 + data_len as usize);
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&file_len.to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes()); // subchunk size
        out.extend_from_slice(&1u16.to_le_bytes()); // PCM
        out.extend_from_slice(&2u16.to_le_bytes()); // stereo
        out.extend_from_slice(&self.sample_rate.to_le_bytes());
        let byte_rate = self.sample_rate * 2 * 2;
        out.extend_from_slice(&byte_rate.to_le_bytes());
        out.extend_from_slice(&4u16.to_le_bytes()); // block align
        out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.to_le_bytes());

        let (pan_l, pan_r) = calculate_pan_weights(pan);

        for &sample in slice {
            let s_l = (sample * gain * pan_l).clamp(-1.0, 1.0);
            let s_r = (sample * gain * pan_r).clamp(-1.0, 1.0);
            let pcm_l = (s_l * 32767.0).round() as i16;
            let pcm_r = (s_r * 32767.0).round() as i16;
            out.extend_from_slice(&pcm_l.to_le_bytes());
            out.extend_from_slice(&pcm_r.to_le_bytes());
        }
        out
    }

    /// Encodes a portion of the track to standard 16-bit PCM WAV for native OS playback at unity mix.
    #[allow(dead_code)]
    pub fn to_wav_bytes(&self, start_secs: f64, end_secs: f64) -> Vec<u8> {
        self.to_wav_bytes_mixed(start_secs, end_secs, 1.0, 0.0)
    }
}

fn compute_waveform_peaks(samples: &[f32], bins: usize) -> Vec<f32> {
    if samples.is_empty() || bins == 0 {
        return vec![0.5; bins.max(1)];
    }
    let samples_per_bin = (samples.len() as f32 / bins as f32).max(1.0);
    let mut peaks = Vec::with_capacity(bins);

    for bin in 0..bins {
        let start = (bin as f32 * samples_per_bin).floor() as usize;
        let end = ((bin + 1) as f32 * samples_per_bin).ceil() as usize;
        let slice = &samples[start.min(samples.len())..end.min(samples.len())];
        let peak = slice.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
        peaks.push(peak.clamp(0.08, 1.0));
    }
    peaks
}

/// Native zero-dependency OS audio playback (asynchronous WinMM on Windows, no-op on non-Windows).
pub struct NativeAudioOutput;

#[cfg(windows)]
static PLAYING_BUFFER: std::sync::Mutex<Option<Vec<u8>>> = std::sync::Mutex::new(None);

impl NativeAudioOutput {
    #[cfg(windows)]
    pub fn play_wav(wav_bytes: &[u8]) {
        #[link(name = "winmm")]
        extern "system" {
            fn PlaySoundW(
                psz_sound: *const u16,
                hmod: *mut core::ffi::c_void,
                fdw_sound: u32,
            ) -> i32;
        }
        const SND_ASYNC: u32 = 0x0001;
        const SND_MEMORY: u32 = 0x0004;
        const SND_NODEFAULT: u32 = 0x0002;
        const SND_PURGE: u32 = 0x0040;

        // Stop prior sound before updating buffer
        unsafe {
            PlaySoundW(core::ptr::null(), core::ptr::null_mut(), SND_PURGE);
        }

        if let Ok(mut guard) = PLAYING_BUFFER.lock() {
            let buffer = wav_bytes.to_vec();
            let ptr = buffer.as_ptr() as *const u16;
            *guard = Some(buffer);
            unsafe {
                PlaySoundW(
                    ptr,
                    core::ptr::null_mut(),
                    SND_ASYNC | SND_MEMORY | SND_NODEFAULT,
                );
            }
        }
    }

    #[cfg(not(windows))]
    pub fn play_wav(_wav_bytes: &[u8]) {}

    #[cfg(windows)]
    pub fn stop() {
        #[link(name = "winmm")]
        extern "system" {
            fn PlaySoundW(
                psz_sound: *const u16,
                hmod: *mut core::ffi::c_void,
                fdw_sound: u32,
            ) -> i32;
        }
        const SND_PURGE: u32 = 0x0040;
        unsafe {
            PlaySoundW(core::ptr::null(), core::ptr::null_mut(), SND_PURGE);
        }
        if let Ok(mut guard) = PLAYING_BUFFER.lock() {
            *guard = None;
        }
    }

    #[cfg(not(windows))]
    pub fn stop() {}
}

/// Calculates stereo pan gain multipliers for left and right channels using equal-power panning.
pub fn calculate_pan_weights(pan: f32) -> (f32, f32) {
    egui_macview::calculate_pan_weights(pan)
}
