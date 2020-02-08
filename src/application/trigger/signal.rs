use super::Channel;
use super::Edge;
use super::Mode;

#[derive(relm_derive::Msg, Clone)]
pub enum Signal {
    Auto,
    Normal,
    Single,
    Mode(Mode),
    Channel(Channel),
    Source(redpitaya_scpi::trigger::Source),
    Edge(Edge),
    InternalTick,
    Redraw(cairo::Context, crate::application::Model),
}
