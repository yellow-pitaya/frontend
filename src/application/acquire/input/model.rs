#[derive(Clone)]
pub struct Model {
    pub attenuation: u8,
    pub started: bool,
    pub source: redpitaya_scpi::acquire::Source,
    pub acquire: redpitaya_scpi::acquire::Acquire,
}
