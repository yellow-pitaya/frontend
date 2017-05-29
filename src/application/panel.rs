pub trait Panel {
    fn draw(&self, context: &::cairo::Context, model: &super::Model);
}
