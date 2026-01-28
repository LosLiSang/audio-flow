pub struct AudioMixer {
    buffer_size: usize,
}

impl AudioMixer {
    pub fn new() -> Self {
        Self { buffer_size: 512 }
    }

    pub fn mix(&self, inputs: &[&[f32]], output: &mut [f32]) {
        output.iter_mut().for_each(|s| *s = 0.0);

        for input in inputs {
            for (i, &sample) in input.iter().enumerate() {
                if i < output.len() {
                    output[i] += sample;
                }
            }
        }

        let num_inputs = inputs.len().max(1) as f32;
        for sample in output.iter_mut() {
            *sample = (*sample / num_inputs).clamp(-1.0, 1.0);
        }
    }

    pub fn db_to_linear(&self, db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    pub fn linear_to_db(&self, linear: f32) -> f32 {
        if linear <= 0.0 {
            return -f32::INFINITY;
        }
        20.0 * linear.log10()
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new()
    }
}
