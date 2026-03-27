use nih-plug::prelude::*;
use std::sync::Arc;

pub struct KuteCompressor {
    pub threshold_db: f32,
    pub ratio: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub makeup_gain_db: f32,

    pub sample_rate: f32,

    // memory state 
    current_gr_db: f32, 
}

impl KuteCompressor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold_db: -20.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 100.0,
            makeup_gain_db: 0.0,
            sample_rate,
            current_gr_db: 0.0,
        }
    }
}

impl KuteCompressor {
    pub fn process_sammple(&mut self, input: f32) -> f32 {
        let input_abs = input.abs();
        let input_db = if input_abs < 0.000001 {
            -120.0
        } else {
            20.0 * input_abs.log10()
        };

        let mut gr_target_db = 0.0;

        if input_db > self.threshold_db {
            let overshoot = input_db - self.threshold_db;
            gr_target_db = -overshoot * (1.0 - (1.0 / self.ratio));
        }

        let attack_sec = self.attack_ms / 1000.0;
        let release_sec = self.release_ms / 1000.0;

        let attack_coef = (-1.0 / (attack_sec * self.sample_rate)).exp();
        let release_coef = (-1.0 / (release_sec * self.sample_rate)).exp();

        let coef = if gr_target_db < self.current_gr_db {
            attack_coef
        } else {
            release_coef
        };

        self.current_gr_db = (coef * self.current_gr_db) + ((1.0 - coef) * gr_target_db);

        let total_gain_db = self.current_gr_db * self.makeup_gain_db;
        let linear_gain = 10.0_f32.powf(total_gain_db / 20.0);

        input * linear_gain
    }
}

#[derive(Parms)]
struct CompressorParams {
    #[id = "threshold"]
    pub threshold: FloatParam,

    #[id = "ratio"]
    pub ratio: FloatParam,

    #[id = "attack"]
    pub attack: FloatParam,

    #[id = "release"]
    pub release: FloatParam,

    #[id = "makeup"]
    pub makeup_gain: FloatParam,
}

impl Default for CompressorParams {
    fn default() -> Self {
        Self {
            threshold: FloatParam::new("Threshold", -20.0, FloatRange::Linear { min: -60.0, max: 0.0 })
                .with_unit(" dB"),
            ratio: FloatParam::new("Ratio", 4.0, FloatRange::Linear { min: 1.0, max: 20.0 }),
            attack: FloatParam::new("Attack", 10.0, FloatRange::Skewed { min: 0.1, max: 100.0, factor: 0.5 })
                .with_unit(" ms"),
            release: FloatParam::new("Release", 100.0, FloatRange::Skewed { min: 5.0, max: 2000.0, factor: 0.5 })
                .with_unit(" ms"),
            makeup_gain: FloatParam::new("Makeup Gain", 0.0, FloatRange::Linear { min: -20.0, max: 20.0 })
                .with_unit(" dB"),
        }
    }
}
