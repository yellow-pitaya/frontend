extern crate conrod;
extern crate glium;

use conrod::backend::glium::glium::Surface;
use glium::DisplayBuild;

fn main() {
    let display = glium::glutin::WindowBuilder::new()
        .with_title("Redpitaya")
        .build_glium()
        .unwrap();

    let mut ui = conrod::UiBuilder::new([400.0, 200.0])
        .build();

    let mut renderer = conrod::backend::glium::Renderer::new(&display)
        .unwrap();

    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

    'main: loop {
        for event in display.poll_events() {
            match event {
                glium::glutin::Event::Closed => break 'main,
                _ => {},
            }
        }

        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display, &mut target, &image_map)
                .unwrap();
            target.finish()
                .unwrap();
        }
    }
}
