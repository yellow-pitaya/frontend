#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub width: i32,
    pub height: i32,
}

#[derive(Copy, Clone, Debug)]
pub struct Scales {
    pub h: (f64, f64),
    pub v: (f64, f64),
    pub n_samples: u32,
    pub window: Rect,
}

impl Scales {
    pub fn get_width(&self) -> f64 {
        self.h.1 - self.h.0
    }

    pub fn get_height(&self) -> f64 {
        self.v.1 - self.v.0
    }

    pub fn from_sampling_rate(&mut self, rate: redpitaya_scpi::acquire::SamplingRate) {
        let duration = rate.get_buffer_duration();
        let h =
            (duration.as_secs() * 1_000_000 + duration.subsec_nanos() as u64 / 1_000) as f64 / 2.0;

        self.h.0 = -h;
        self.h.1 = h;
    }

    pub fn v_div(&self) -> f64 {
        (self.v.1 - self.v.0) / 10.0
    }

    pub fn h_div(&self) -> f64 {
        (self.h.1 - self.h.0) / 10.0
    }

    pub fn sample_to_ms(&self, sample: u32) -> f64 {
        sample as f64 / self.n_samples as f64 * (self.h.1 - self.h.0) + self.h.0
    }

    pub fn x_to_offset(&self, x: i32) -> f64 {
        x as f64 / self.window.width as f64 * self.get_width() + self.h.0
    }

    pub fn y_to_offset(&self, y: i32) -> f64 {
        y as f64 / -self.window.height as f64 * self.get_height() + self.v.1
    }
}
