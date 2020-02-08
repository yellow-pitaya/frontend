use super::Mode;

#[derive(Clone)]
pub struct Model {
    pub trigger: redpitaya_scpi::trigger::Trigger,
    pub channel: Option<super::Channel>,
    pub edge: Option<super::Edge>,
    pub mode: Mode,
}
