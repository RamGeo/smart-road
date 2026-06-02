use std::f32::consts::PI;
use std::sync::Mutex;

use sdl2::audio::{AudioQueue, AudioSpecDesired};

const SAMPLE_RATE: u32 = 44_100;

pub struct Audio {
    device: Mutex<AudioQueue<f32>>,
}

impl Audio {
    pub fn try_init(sdl: &sdl2::Sdl) -> Option<Self> {
        let audio_sub = sdl.audio().ok()?;
        let desired = AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(1),
            samples: None,
        };
        let device = audio_sub.open_queue(None, &desired).ok()?;
        device.resume();
        Some(Audio {
            device: Mutex::new(device),
        })
    }

    /// Short low "pop" — distinct from the R-toggle chirps.
    pub fn play_spawn(&self) {
        self.queue_tones(&[(220.0, 35), (165.0, 45)]);
    }

    pub fn play_random_on(&self) {
        self.queue_tones(&[
            (520.0, 90),
            (1_040.0, 90),
            (780.0, 60),
            (1_560.0, 60),
        ]);
    }

    pub fn play_random_off(&self) {
        self.queue_tones(&[(880.0, 80), (440.0, 80)]);
    }

    fn queue_tones(&self, tones: &[(f32, u32)]) {
        let mut samples = Vec::new();
        for &(freq, ms) in tones {
            samples.extend(synth_beep(freq, ms, SAMPLE_RATE));
        }
        if let Ok(device) = self.device.lock() {
            let _ = device.queue_audio(&samples);
        }
    }
}

fn synth_beep(freq: f32, duration_ms: u32, sample_rate: u32) -> Vec<f32> {
    let count = sample_rate * duration_ms / 1000;
    (0..count)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let progress = i as f32 / count as f32;
            let envelope = (1.0 - progress).powf(0.4);
            envelope * 0.35 * (2.0 * PI * freq * t).sin()
        })
        .collect()
}
