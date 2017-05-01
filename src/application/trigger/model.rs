use super::Mode;

#[derive(Clone)]
pub struct Model {
    pub trigger: ::redpitaya_scpi::trigger::Trigger,
    pub mode: Mode,
}
