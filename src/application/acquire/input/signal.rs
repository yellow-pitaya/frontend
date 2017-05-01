#[derive(Msg)]
pub enum Signal {
    Attenuation(u8),
    Data,
    Gain(::redpitaya_scpi::acquire::Gain),
    Level(u32),
}
