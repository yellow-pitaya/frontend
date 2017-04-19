use gtk::{
    BoxExt,
    ButtonExt,
    ContainerExt,
    RangeExt,
    ToggleButtonExt,
    WidgetExt,
};

#[derive(Clone)]
pub struct Application {
    window: ::gtk::Window,
    drawing_area: ::gtk::DrawingArea,
    acquire_toggle: ::gtk::ToggleButton,
    generator_toggle: ::gtk::ToggleButton,
    duty_cycle_scale: ::gtk::Scale,
    redpitaya: ::redpitaya_scpi::Redpitaya,
    scales: [(f64, f64); 2],
}

impl Application {
    pub fn run() {
        ::relm::run::<Self>()
            .unwrap();
    }

    pub fn draw(&self) {
        let width = self.drawing_area.get_allocated_width() as f64;
        let height = self.drawing_area.get_allocated_height() as f64;
        let context = self.create_context();

        context.set_source_rgb(1.0, 1.0, 1.0);
        context.rectangle(0.0, 0.0, width, height);
        context.fill();

        self.draw_scales(&context, width, height);
        if self.redpitaya.acquire.is_started() {
            self.transform(&context, width, height);
            self.draw_data(&context);
        }
    }

    fn create_context(&self) -> ::cairo::Context {
        let window = self.drawing_area.get_window().unwrap();

        unsafe {
            use ::glib::translate::ToGlibPtr;

            let context = ::gdk_sys::gdk_cairo_create(window.to_glib_none().0);

            ::std::mem::transmute(context)
        }
    }

    fn transform(&self, context: &::cairo::Context, width: f64, height: f64) {
        context.set_line_width(0.1);
        context.set_source_rgb(0.0, 0.0, 0.0);

        context.scale(width / self.scales[0].1, height / (self.scales[1].0.abs() + self.scales[1].1.abs()));
        context.translate(self.scales[0].0, self.scales[1].1);
    }

    fn draw_scales(&self, context: &::cairo::Context, width: f64, height: f64) {
        context.set_line_width(1.0);
        context.set_source_rgb(0.0, 0.0, 0.0);

        context.rectangle(0.0, 0.0, width, height);
        context.stroke();

        for i in 1..10 {
            let x = width / 10.0 * (i as f64);

            if i != 5 {
                context.set_source_rgba(0.0, 0.0, 0.0, 0.2);
            } else {
                context.set_source_rgb(0.0, 0.0, 0.0);
            }

            context.move_to(x, 0.0);
            context.line_to(x, height);

            let y = height / 10.0 * (i as f64);

            context.move_to(0.0, y);
            context.line_to(width, y);

            context.stroke();
        }
    }

    fn draw_data(&self, context: &::cairo::Context) {
        let message = self.redpitaya.data.read_all(::redpitaya_scpi::acquire::Source::IN1);

        let mut data = message
            .trim_matches(|c: char| c == '{' || c == '}' || c == '!' || c.is_alphabetic())
            .split(",")
            .map(|s| {
                match s.parse::<f64>() {
                    Ok(f) => f,
                    Err(_) => {
                        error!("Invalid data '{}'", s);
                        0.0
                    },
                }
            });

        context.set_source_rgb(1.0, 0.0, 0.0);

        for x in 0..16384 {
            match data.next() {
                Some(y) => {
                    context.line_to(x as f64, y);
                    context.move_to(x as f64, y);
                },
                None => (),
            }
        }
        context.stroke();
    }
}

#[derive(Clone)]
pub enum Signal {
    AcquireToggle,
    GeneratorAmplitude(::redpitaya_scpi::generator::Source, f32),
    GeneratorFrequency(::redpitaya_scpi::generator::Source, u32),
    GeneratorDutyCycle(::redpitaya_scpi::generator::Source, u32),
    Draw,
    GeneratorToggle(::redpitaya_scpi::generator::Source),
    GeneratorSignal(::redpitaya_scpi::generator::Source, ::redpitaya_scpi::generator::Form),
    Quit,
}

impl ::relm::DisplayVariant for Signal {
    fn display_variant(&self) -> &'static str {
        match *self {
            Signal::AcquireToggle => "Signal::AcquireToggle",
            Signal::GeneratorAmplitude(_, _) => "Signal::GeneratorAmplitude",
            Signal::GeneratorFrequency(_, _) => "Signal::GeneratorFrequency",
            Signal::GeneratorDutyCycle(_, _) => "Signal::GeneratorDutyCycle",
            Signal::Draw => "Signal::Draw",
            Signal::GeneratorToggle(_) => "Signal::GeneratorToggle",
            Signal::GeneratorSignal(_, _) => "Signal::GeneratorSignal",
            Signal::Quit => "Signal::Quit",
        }
    }
}

impl ::relm::Widget for Application {
    type Model = ();
    type Msg = Signal;
    type Root = ::gtk::Window;

    fn model() -> Self::Model {
    }

    fn root(&self) -> &Self::Root {
        &self.window
    }

    fn update(&mut self, event: Signal, _: &mut Self::Model) {
        match event {
            Signal::AcquireToggle => if self.redpitaya.acquire.is_started() {
                self.redpitaya.acquire.stop();
                self.acquire_toggle.set_label("Run");

            } else {
                self.redpitaya.acquire.start();
                self.acquire_toggle.set_label("Stop");
            },
            Signal::GeneratorAmplitude(source, value) => self.redpitaya.generator.set_amplitude(source, value),
            Signal::GeneratorFrequency(source, value) => self.redpitaya.generator.set_frequency(source, value),
            Signal::GeneratorDutyCycle(source, value) => self.redpitaya.generator.set_duty_cycle(source, value),
            Signal::Draw => self.draw(),
            Signal::GeneratorToggle(source) => if self.redpitaya.generator.is_started(source) {
                self.redpitaya.generator.stop(source);
                self.generator_toggle.set_label("Run");
            } else {
                self.redpitaya.generator.start(source);
                self.generator_toggle.set_label("Stop");
            },
            Signal::GeneratorSignal(source, form) => {
                self.redpitaya.generator.set_form(source, form);

                let is_pwm = form == ::redpitaya_scpi::generator::Form::PWM;
                self.duty_cycle_scale.set_visible(is_pwm);
            },
            Signal::Quit => {
                self.redpitaya.acquire.stop();
                self.redpitaya.generator.stop(::redpitaya_scpi::generator::Source::OUT1);
                self.redpitaya.generator.stop(::redpitaya_scpi::generator::Source::OUT2);
                ::gtk::main_quit();
            },
        };
    }

    fn view(relm: ::relm::RemoteRelm<Signal>, _: &Self::Model) -> Self {
        // @TODO use program arguments
        let redpitaya = ::redpitaya_scpi::Redpitaya::new("192.168.1.5:5000");

        let main_box = ::gtk::Box::new(::gtk::Orientation::Horizontal, 0);

        let drawing_area = ::gtk::DrawingArea::new();
        main_box.pack_start(&drawing_area, true, true, 0);

        let stream = relm.stream().clone();
        drawing_area.connect_draw(move |_, _| {
            stream.emit(Signal::Draw);

            ::gtk::Inhibit(false)
        });

        let stream = relm.stream().clone();
        GLOBAL.with(move |global| {
            *global.borrow_mut() = Some(stream)
        });

        ::gtk::timeout_add(1_000, || {
            GLOBAL.with(|global| {
                if let Some(ref stream) = *global.borrow() {
                    stream.emit(Signal::Draw);
                }
            });

            ::glib::Continue(true)
        });

        let acquire_page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);

        let acquire_toggle = ::gtk::ToggleButton::new_with_label("Run");
        acquire_page.pack_start(&acquire_toggle, false, false, 0);
        connect!(relm, acquire_toggle, connect_toggled(_), Signal::AcquireToggle);

        let generator_page = ::gtk::Box::new(::gtk::Orientation::Vertical, 0);

        let generator_toggle = ::gtk::ToggleButton::new_with_label("Run");
        generator_page.pack_start(&generator_toggle, false, false, 0);
        connect!(relm,generator_toggle, connect_toggled(_), Signal::GeneratorToggle(::redpitaya_scpi::generator::Source::OUT1));

        let forms = vec![
            ::redpitaya_scpi::generator::Form::SINE,
            ::redpitaya_scpi::generator::Form::SQUARE,
            ::redpitaya_scpi::generator::Form::TRIANGLE,
            ::redpitaya_scpi::generator::Form::SAWU,
            ::redpitaya_scpi::generator::Form::SAWD,
            ::redpitaya_scpi::generator::Form::PWM,
            // @TODO ::redpitaya_scpi::generator::Form::ARBITRARY,
        ];

        let mut group_member = None;

        for form in forms {
            let button = ::gtk::RadioButton::new_with_label_from_widget(
                group_member.as_ref(),
                format!("{}", form).as_str()
            );
            generator_page.pack_start(&button, false, true, 0);

            let stream = relm.stream().clone();
            button.connect_toggled(move |f| {
                if f.get_active() {
                    stream.emit(
                        Signal::GeneratorSignal(::redpitaya_scpi::generator::Source::OUT1, form.clone())
                    );
                }
            });

            if group_member == None {
                group_member = Some(button);
            }
        }

        let adjustement = ::gtk::Adjustment::new(
            redpitaya.generator.get_amplitude(::redpitaya_scpi::generator::Source::OUT1) as f64,
            -1.0,
            1.0,
            0.01,
            0.1,
            0.0
        );
        let button = ::gtk::Scale::new(::gtk::Orientation::Horizontal, Some(&adjustement));
        let stream = relm.stream().clone();
        button.connect_change_value(move |_, _, value| {
            stream.emit(Signal::GeneratorAmplitude(::redpitaya_scpi::generator::Source::OUT1, value as f32));

            ::gtk::Inhibit(false)
        });
        generator_page.pack_start(&button, false, true, 0);

        let adjustement = ::gtk::Adjustment::new(
            redpitaya.generator.get_frequency(::redpitaya_scpi::generator::Source::OUT1) as f64,
            0.0,
            62_500_000.0,
            1.0,
            1_000.0,
            0.0
        );
        let button = ::gtk::Scale::new(::gtk::Orientation::Horizontal, Some(&adjustement));
        let stream = relm.stream().clone();
        button.connect_change_value(move |_, _, value| {
            stream.emit(Signal::GeneratorFrequency(::redpitaya_scpi::generator::Source::OUT1, value as u32));

            ::gtk::Inhibit(false)
        });
        generator_page.pack_start(&button, false, true, 0);

        let adjustement = ::gtk::Adjustment::new(
            redpitaya.generator.get_duty_cycle(::redpitaya_scpi::generator::Source::OUT1) as f64,
            0.0,
            100.0,
            1.0,
            10.0,
            0.0
        );
        let duty_cycle_scale = ::gtk::Scale::new(::gtk::Orientation::Horizontal, Some(&adjustement));
        let stream = relm.stream().clone();
        duty_cycle_scale.connect_change_value(move |_, _, value| {
            stream.emit(Signal::GeneratorDutyCycle(::redpitaya_scpi::generator::Source::OUT1, value as u32));

            ::gtk::Inhibit(false)
        });
        generator_page.pack_start(&duty_cycle_scale, false, true, 0);

        let notebook = ::gtk::Notebook::new();
        notebook.append_page(
            &acquire_page,
            Some(&::gtk::Label::new(Some("Acquire")))
        );
        notebook.append_page(
            &generator_page,
            Some(&::gtk::Label::new(Some("Generator")))
        );
        main_box.pack_start(&notebook, false, true, 0);

        let window = ::gtk::Window::new(::gtk::WindowType::Toplevel);
        window.add(&main_box);
        connect!(relm, window, connect_destroy(_), Signal::Quit);

        window.show_all();
        duty_cycle_scale.set_visible(false);

        Application {
            window: window,
            drawing_area: drawing_area,
            acquire_toggle: acquire_toggle,
            generator_toggle: generator_toggle,
            duty_cycle_scale: duty_cycle_scale,
            redpitaya: redpitaya,
            scales: [
                (0.0, 16384.0),
                (-5.0, 5.0),
            ],
        }
    }
}

thread_local!(
    static GLOBAL: ::std::cell::RefCell<Option<::relm::EventStream<Signal>>> = ::std::cell::RefCell::new(None)
);
