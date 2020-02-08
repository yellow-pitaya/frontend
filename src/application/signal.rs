#[derive(relm_derive::Msg, Clone)]
pub enum Signal {
    AcquireRate(redpitaya_scpi::acquire::SamplingRate),
    GraphDraw,
    Level(String, i32),
    NeedDraw,
    Resize(i32, i32),
    TriggerAuto,
    TriggerNormal,
    TriggerSingle,
    Quit,
}
