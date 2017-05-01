#[derive(Clone)]
pub struct Model {
    pub source: ::redpitaya_scpi::generator::Source,
    pub generator: ::redpitaya_scpi::generator::Generator,
}
