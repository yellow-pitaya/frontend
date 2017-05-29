use super::Channel;
use super::Edge;
use super::Mode;

#[derive(Msg)]
pub enum Signal {
    Auto,
    Normal,
    Single,
    Mode(Mode),
    Channel(Channel),
    Source(::redpitaya_scpi::trigger::Source),
    Edge(Edge),
    InternalTick,
    Delay(u8),
}
