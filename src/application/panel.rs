use gtk::WidgetExt;

pub trait Panel {
    fn draw(&self, context: &::cairo::Context, model: &super::Model);

    fn create_context(&self, widget: &::gtk::DrawingArea) -> ::cairo::Context {
        let window = widget.get_window().unwrap();

        unsafe {
            use ::glib::translate::ToGlibPtr;

            let context = ::gdk_sys::gdk_cairo_create(window.to_glib_none().0);

            ::std::mem::transmute(context)
        }
    }
}
