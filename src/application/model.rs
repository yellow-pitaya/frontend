#[derive(Clone)]
pub struct Model {
    pub rate: redpitaya_scpi::acquire::SamplingRate,
    pub redpitaya: redpitaya_scpi::Redpitaya,
    pub scales: crate::Scales,
    pub levels: std::collections::HashMap<String, i32>,
}

impl Model {
    pub fn offset<D>(&self, channel: D) -> f64 where D: std::fmt::Display {
        let channel = format!("{}", channel);

        let level = match self.levels.get(&channel) {
            Some(level) => if channel == "DELAY" {
                self.scales.x_to_offset(*level)
            }
            else {
                self.scales.y_to_offset(*level)
            },
            None => 0.0,
        };

        level
    }
}
