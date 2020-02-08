use application::{
    graph,
    acquire,
    generator,
    trigger,
};
use gtk::{
    BoxExt,
    ContainerExt,
    NotebookExt,
    StatusbarExt,
    WidgetExt,
    GtkWindowExt,
    prelude::NotebookExtManual,
};
use relm::ContainerWidget;

macro_rules! redraw {
    ($self:ident, $widget:ident, $image:ident) => {
        let context = ::cairo::Context::new(&$image);

        $self.transform($self.model.scales, &context, $image.get_width() as f64, $image.get_height() as f64);
        context.set_line_width(0.01);

        $self.$widget.emit(super::$widget::Signal::Redraw(context.clone(), $self.model.clone()));
    }
}

#[derive(Clone)]
pub struct Widget {
    model: super::Model,
    relm: ::relm::Relm<Self>,
    window: ::gtk::Window,
    graph: ::relm::Component<graph::Widget>,
    status_bar: ::gtk::Statusbar,
    acquire: ::relm::Component<acquire::Widget>,
    generator: ::relm::Component<generator::Widget>,
    trigger: ::relm::Component<trigger::Widget>,
}

impl Widget {
    fn update_status(&self) {
        let status = format!(
            "{} - {} V/div - {} µs/div",
            self.model.rate,
            self.model.scales.v_div(),
            self.model.scales.h_div()
        );

        self.status_bar.push(
            self.status_bar.get_context_id("sampling-rate"),
            status.as_str()
        );
    }

    fn draw(&mut self) {
        self.update_status();

        let image = ::cairo::ImageSurface::create(
            ::cairo::Format::ARgb32,
            self.model.scales.window.width,
            self.model.scales.window.height
        ).unwrap();

        redraw!(self, graph, image);
        redraw!(self, trigger, image);
        redraw!(self, generator, image);
        redraw!(self, acquire, image);

        self.graph.emit(super::graph::Signal::SetImage(image));
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

impl ::relm::Update for Widget {
    type Model = super::Model;
    type ModelParam = ::redpitaya_scpi::Redpitaya;
    type Msg = super::Signal;

    fn model(_: &::relm::Relm<Self>, redpitaya: Self::ModelParam) -> Self::Model {
        let mut scales = ::Scales {
            h: (0.0, 0.0),
            v: (-5.0, 5.0),
            n_samples: redpitaya.data.buffer_size().unwrap(),
            window: ::scales::Rect { width: 0, height: 0 },
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

    fn update(&mut self, event: super::Signal) {
        match event {
            super::Signal::AcquireRate(rate) => {
                self.model.rate = rate;
                self.model.scales.from_sampling_rate(rate);
                self.graph.emit(graph::Signal::Invalidate);
            },

            super::Signal::NeedDraw => self.graph.emit(graph::Signal::Invalidate),
            super::Signal::GraphDraw => self.draw(),
            super::Signal::Level(channel, level) => {
                self.model.levels.insert(channel, level);
            },
            super::Signal::Resize(width, height) => {
                self.model.scales.window.width = width;
                self.model.scales.window.height = height;
                self.relm.stream().emit(super::Signal::NeedDraw)
            },

            super::Signal::TriggerAuto | super::Signal::TriggerSingle => {
                self.acquire.emit(acquire::Signal::SetData(
                    ::redpitaya_scpi::acquire::Source::IN1,
                    self.model.redpitaya.data.read_all(::redpitaya_scpi::acquire::Source::IN1)
                ));
                self.acquire.emit(acquire::Signal::SetData(
                    ::redpitaya_scpi::acquire::Source::IN2,
                    self.model.redpitaya.data.read_all(::redpitaya_scpi::acquire::Source::IN2)
                ));
            },
            super::Signal::TriggerNormal => {
                self.acquire.emit(acquire::Signal::SetData(
                    ::redpitaya_scpi::acquire::Source::IN1,
                    self.model.redpitaya.data.read_oldest(::redpitaya_scpi::acquire::Source::IN1, 16_384)
                ));
                self.acquire.emit(acquire::Signal::SetData(
                    ::redpitaya_scpi::acquire::Source::IN2,
                    self.model.redpitaya.data.read_oldest(::redpitaya_scpi::acquire::Source::IN2, 16_384)
                ));
            },
            super::Signal::Quit => {
                self.model.redpitaya.acquire.stop();
                self.model.redpitaya.generator.stop(::redpitaya_scpi::generator::Source::OUT1);
                self.model.redpitaya.generator.stop(::redpitaya_scpi::generator::Source::OUT2);
                ::gtk::main_quit();
            },
        };
    }
}

impl ::relm::Widget for Widget {
    type Root = ::gtk::Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &::relm::Relm<Self>, model: Self::Model) -> Self {
        let main_box = ::gtk::Box::new(::gtk::Orientation::Horizontal, 0);

        let graph_page = ::gtk::EventBox::new();
        main_box.pack_start(&graph_page, true, true, 0);

        let graph = graph_page.add_widget::<graph::Widget>(());
        connect!(graph@graph::Signal::Draw, relm, super::Signal::GraphDraw);
        connect!(graph@graph::Signal::Level(ref channel, offset), relm, super::Signal::Level(channel.clone(), offset));
        connect!(graph@graph::Signal::Resize(w, h), relm, super::Signal::Resize(w, h));

        let vbox = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        main_box.pack_start(&vbox, false, false, 0);

        let notebook = ::gtk::Notebook::new();
        notebook.set_scrollable(true);
        vbox.pack_start(&notebook, true, true, 0);

        let acquire_page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        acquire_page.set_border_width(10);

        let acquire = acquire_page.add_widget::<acquire::Widget>(model.redpitaya.acquire.clone());
        connect!(acquire@acquire::Signal::Rate(rate), relm, super::Signal::AcquireRate(rate));
        connect!(acquire@acquire::Signal::Start(source), graph, graph::Signal::SourceStart(super::graph::level::widget::Orientation::Left, format!("{}", source)));
        connect!(acquire@acquire::Signal::Stop(source), graph, graph::Signal::SourceStop(super::graph::level::widget::Orientation::Left, format!("{}", source)));

        notebook.append_page(
            &acquire_page,
            Some(&::gtk::Label::new(Some("Acquire")))
        );

        let scrolled_window = ::gtk::ScrolledWindow::new::<::gtk::Adjustment, ::gtk::Adjustment>(None, None);
        scrolled_window.set_border_width(10);
        notebook.append_page(
            &scrolled_window,
            Some(&::gtk::Label::new(Some("Generator")))
        );

        let generator = scrolled_window.add_widget::<generator::Widget>(model.redpitaya.generator.clone());
        connect!(generator@generator::Signal::Amplitude(_, _), relm, super::Signal::NeedDraw);
        connect!(generator@generator::Signal::DutyCycle(_, _), relm, super::Signal::NeedDraw);
        connect!(generator@generator::Signal::Frequency(_, _), relm, super::Signal::NeedDraw);
        connect!(generator@generator::Signal::Offset(_, _), relm, super::Signal::NeedDraw);
        connect!(generator@generator::Signal::Form(_, _), relm, super::Signal::NeedDraw);
        connect!(generator@generator::Signal::Start(source), graph, graph::Signal::SourceStart(super::graph::level::widget::Orientation::Left, format!("{}", source)));
        connect!(generator@generator::Signal::Stop(source), graph, graph::Signal::SourceStop(super::graph::level::widget::Orientation::Left, format!("{}", source)));

        let status_bar = ::gtk::Statusbar::new();
        vbox.pack_start(&status_bar, false, true, 0);

        let window = ::gtk::Window::new(::gtk::WindowType::Toplevel);
        window.set_title("Yellow Pitaya");
        window.add(&main_box);
        connect!(relm, window, connect_destroy(_), super::Signal::Quit);

        let trigger_page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);
        trigger_page.set_border_width(10);
        let trigger = trigger_page.add_widget::<trigger::Widget>(model.redpitaya.trigger.clone());
        connect!(trigger@trigger::Signal::Auto, relm, super::Signal::TriggerAuto);
        connect!(trigger@trigger::Signal::Normal, relm, super::Signal::TriggerNormal);
        connect!(trigger@trigger::Signal::Single, relm, super::Signal::TriggerSingle);

        connect!(trigger@trigger::Signal::Auto, graph, graph::Signal::SourceStop(super::graph::level::widget::Orientation::Right, "TRIG".to_owned()));
        connect!(trigger@trigger::Signal::Normal, graph, graph::Signal::SourceStart(super::graph::level::widget::Orientation::Right, "TRIG".to_owned()));
        connect!(trigger@trigger::Signal::Single, graph, graph::Signal::SourceStart(super::graph::level::widget::Orientation::Right, "TRIG".to_owned()));

        connect!(trigger@trigger::Signal::Auto, graph, graph::Signal::SourceStop(super::graph::level::widget::Orientation::Top, "DELAY".to_owned()));
        connect!(trigger@trigger::Signal::Normal, graph, graph::Signal::SourceStart(super::graph::level::widget::Orientation::Top, "DELAY".to_owned()));
        connect!(trigger@trigger::Signal::Single, graph, graph::Signal::SourceStart(super::graph::level::widget::Orientation::Top, "DELAY".to_owned()));

        notebook.append_page(
            &trigger_page,
            Some(&::gtk::Label::new(Some("Trigger")))
        );

        Widget {
            relm: relm.clone(),
            model,
            window: window,
            graph: graph,
            status_bar: status_bar,
            acquire: acquire,
            generator: generator,
            trigger: trigger,
        }
    }

    fn init_view(&mut self) {
        self.model.redpitaya.data.set_units(::redpitaya_scpi::data::Unit::VOLTS);

        self.window.show_all();

        self.model.redpitaya.acquire.start();

        // @FIXME
        // self.trigger.widget().single_button.set_visible(false);
    }
}
