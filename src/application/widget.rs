use application::{
    graph,
    acquire,
    generator,
    trigger,
};
use gtk::{
    BoxExt,
    ContainerExt,
    WidgetExt,
    WindowExt,
};
use relm::ContainerWidget;

#[derive(Clone)]
pub struct Widget {
    window: ::gtk::Window,
    graph: ::relm::Component<graph::Widget>,
    status_bar: ::gtk::Statusbar,
    acquire: ::relm::Component<acquire::Widget>,
    generator: ::relm::Component<generator::Widget>,
    trigger: ::relm::Component<trigger::Widget>,
}

impl Widget {
    fn update_status(&self, model: &super::Model) {
        let status = format!(
            "{} - {} V/div - {} µs/div",
            model.rate,
            model.scales.v_div(),
            model.scales.h_div()
        );

        self.status_bar.push(
            self.status_bar.get_context_id("sampling-rate"),
            status.as_str()
        );
    }

    fn draw(&self, model: &mut super::Model) {
        let graph = self.graph.widget();
        let width = graph.get_width();
        let height = graph.get_height();

        model.scales.window.width = width;
        model.scales.window.height = height;

        self.update_status(model);

        let image = ::cairo::ImageSurface::create(::cairo::Format::ARgb32, width as i32, height as i32);

        self.draw_panel(self.graph.widget(), &model, &image);
        self.draw_panel(self.trigger.widget(), &model, &image);
        self.draw_panel(self.generator.widget(), &model, &image);
        self.draw_panel(self.acquire.widget(), &model, &image);

        graph.set_image(&image);
    }

    fn draw_panel(&self, panel: &super::Panel, model: &super::Model, image: &::cairo::ImageSurface) {
        let context = ::cairo::Context::new(image);

        self.transform(model.scales, &context, image.get_width() as f64, image.get_height() as f64);
        context.set_line_width(0.01);
        panel.draw(&context, model);
    }

    fn transform(&self, scales: ::Scales, context: &::cairo::Context, width: f64, height: f64) {
        context.set_matrix(::cairo::Matrix {
            xx: width / scales.get_width(),
            xy: 0.0,
            yy: -height / scales.get_height(),
            yx: 0.0,
            x0: scales.h.1 * width / scales.get_width(),
            y0: scales.v.1 * height / scales.get_height(),
        });
    }
}

impl ::relm::Widget for Widget {
    type Model = super::Model;
    type ModelParam = ::redpitaya_scpi::Redpitaya;
    type Msg = super::Signal;
    type Root = ::gtk::Window;

    fn model(redpitaya: Self::ModelParam) -> Self::Model {
        let mut scales = ::Scales {
            h: (0.0, 0.0),
            v: (-5.0, 5.0),
            n_samples: redpitaya.data.buffer_size().unwrap(),
            window: ::scales::Rect { width: 0.0, height: 0.0 },
        };

        let rate = redpitaya.acquire.get_decimation()
            .unwrap()
            .into();
        scales.from_sampling_rate(rate);

        super::Model {
            rate: rate,
            redpitaya: redpitaya,
            scales: scales,
            levels: ::std::collections::HashMap::new(),
        }
    }

    fn root(&self) -> &Self::Root {
        &self.window
    }

    fn update(&mut self, event: super::Signal, model: &mut Self::Model) {
        match event {
            super::Signal::AcquireRate(rate) => {
                model.rate = rate;
                model.scales.from_sampling_rate(rate);
                self.graph.widget().invalidate();
            },

            super::Signal::NeedDraw => self.graph.widget().invalidate(),
            super::Signal::GraphDraw => self.draw(model),
            super::Signal::Level(channel, level) => {
                model.levels.insert(channel, level);
            },

            super::Signal::TriggerAuto | super::Signal::TriggerSingle => {
                self.acquire.widget().set_data(
                    ::redpitaya_scpi::acquire::Source::IN1,
                    model.redpitaya.data.read_all(::redpitaya_scpi::acquire::Source::IN1)
                );
                self.acquire.widget().set_data(
                    ::redpitaya_scpi::acquire::Source::IN2,
                    model.redpitaya.data.read_all(::redpitaya_scpi::acquire::Source::IN2)
                );
            },
            super::Signal::TriggerNormal => {
                self.acquire.widget().set_data(
                    ::redpitaya_scpi::acquire::Source::IN1,
                    model.redpitaya.data.read_oldest(::redpitaya_scpi::acquire::Source::IN1, 16_384)
                );
                self.acquire.widget().set_data(
                    ::redpitaya_scpi::acquire::Source::IN2,
                    model.redpitaya.data.read_oldest(::redpitaya_scpi::acquire::Source::IN2, 16_384)
                );
            },
            super::Signal::Quit => {
                model.redpitaya.acquire.stop();
                model.redpitaya.generator.stop(::redpitaya_scpi::generator::Source::OUT1);
                model.redpitaya.generator.stop(::redpitaya_scpi::generator::Source::OUT2);
                ::gtk::main_quit();
            },
        };
    }

    fn view(relm: &::relm::RemoteRelm<Self>, model: &Self::Model) -> Self {
        let main_box = ::gtk::Box::new(::gtk::Orientation::Horizontal, 0);

        let graph_page = ::gtk::EventBox::new();
        main_box.pack_start(&graph_page, true, true, 0);

        let graph = graph_page.add_widget::<graph::Widget, _>(&relm, ());
        connect!(graph@graph::Signal::Draw, relm, super::Signal::GraphDraw);
        connect!(graph@graph::Signal::Level(channel, offset), relm, super::Signal::Level(channel, offset));

        let vbox = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        main_box.pack_start(&vbox, false, false, 0);

        let notebook = ::gtk::Notebook::new();
        vbox.pack_start(&notebook, true, true, 0);

        let acquire_page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        acquire_page.set_border_width(10);

        let acquire = acquire_page.add_widget::<acquire::Widget, _>(&relm, model.redpitaya.acquire.clone());
        connect!(acquire@acquire::Signal::Data(_), relm, super::Signal::NeedDraw);
        connect!(acquire@acquire::Signal::Rate(rate), relm, super::Signal::AcquireRate(rate));

        {
            let level_left = graph.widget().level_left();
            connect!(acquire@acquire::Signal::Start(source), level_left, graph::level::Signal::SourceStart(format!("{}", source)));
            connect!(acquire@acquire::Signal::Stop(source), level_left, graph::level::Signal::SourceStop(format!("{}", source)));
        }

        notebook.append_page(
            &acquire_page,
            Some(&::gtk::Label::new(Some("Acquire")))
        );

        let scrolled_window = ::gtk::ScrolledWindow::new(None, None);
        scrolled_window.set_border_width(10);
        notebook.append_page(
            &scrolled_window,
            Some(&::gtk::Label::new(Some("Generator")))
        );

        let generator = scrolled_window.add_widget::<generator::Widget, _>(&relm, model.redpitaya.generator.clone());
        connect!(generator@generator::Signal::Amplitude(_, _), relm, super::Signal::NeedDraw);
        connect!(generator@generator::Signal::DutyCycle(_, _), relm, super::Signal::NeedDraw);
        connect!(generator@generator::Signal::Frequency(_, _), relm, super::Signal::NeedDraw);
        connect!(generator@generator::Signal::Offset(_, _), relm, super::Signal::NeedDraw);
        connect!(generator@generator::Signal::Form(_, _), relm, super::Signal::NeedDraw);

        {
            let level_left = graph.widget().level_left();
            connect!(generator@generator::Signal::Start(source), level_left, graph::level::Signal::SourceStart(format!("{}", source)));
            connect!(generator@generator::Signal::Stop(source), level_left, graph::level::Signal::SourceStop(format!("{}", source)));
        }

        let status_bar = ::gtk::Statusbar::new();
        vbox.pack_start(&status_bar, false, true, 0);

        let window = ::gtk::Window::new(::gtk::WindowType::Toplevel);
        window.set_title("Yellow Pitaya");
        window.add(&main_box);
        connect!(relm, window, connect_destroy(_), super::Signal::Quit);

        let trigger_page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        trigger_page.set_border_width(10);
        let trigger = trigger_page.add_widget::<trigger::Widget, _>(&relm, model.redpitaya.trigger.clone());
        connect!(trigger@trigger::Signal::Auto, relm, super::Signal::TriggerAuto);
        connect!(trigger@trigger::Signal::Normal, relm, super::Signal::TriggerNormal);
        connect!(trigger@trigger::Signal::Single, relm, super::Signal::TriggerSingle);

        {
            let level_right = graph.widget().level_right();

            connect!(trigger@trigger::Signal::Auto, level_right, graph::level::Signal::SourceStop("TRIG".to_owned()));
            connect!(trigger@trigger::Signal::Normal, level_right, graph::level::Signal::SourceStart("TRIG".to_owned()));
            connect!(trigger@trigger::Signal::Single, level_right, graph::level::Signal::SourceStart("TRIG".to_owned()));
        }

        {
            let level_top = graph.widget().level_top();

            connect!(trigger@trigger::Signal::Auto, level_top, graph::level::Signal::SourceStop("DELAY".to_owned()));
            connect!(trigger@trigger::Signal::Normal, level_top, graph::level::Signal::SourceStart("DELAY".to_owned()));
            connect!(trigger@trigger::Signal::Single, level_top, graph::level::Signal::SourceStart("DELAY".to_owned()));
        }

        notebook.append_page(
            &trigger_page,
            Some(&::gtk::Label::new(Some("Trigger")))
        );

        Widget {
            window: window,
            graph: graph,
            status_bar: status_bar,
            acquire: acquire,
            generator: generator,
            trigger: trigger,
        }
    }

    fn init_view(&self, model: &mut super::Model) {
        model.redpitaya.data.set_units(::redpitaya_scpi::data::Unit::VOLTS);

        self.window.show_all();

        model.redpitaya.acquire.start();

        // @FIXME
        self.trigger.widget().single_button.set_visible(false);
    }
}
