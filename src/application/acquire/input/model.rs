#[derive(Clone)]
pub struct Model {
    pub source: ::redpitaya_scpi::acquire::Source,
    pub acquire: ::redpitaya_scpi::acquire::Acquire,
}
