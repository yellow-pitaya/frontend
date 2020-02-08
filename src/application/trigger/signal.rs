use super::Channel;
use super::Edge;
use super::Mode;

#[derive(Msg, Clone)]
pub enum Signal {
    Auto,
    Normal,
    Single,
    Mode(Mode),
    Channel(Channel),
    Source(::redpitaya_scpi::trigger::Source),
    Edge(Edge),
    InternalTick,
    Redraw(::cairo::Context, ::application::Model),
}
