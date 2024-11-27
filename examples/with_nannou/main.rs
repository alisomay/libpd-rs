#![allow(dead_code)]
mod bubble;

use bubble::Bubble;
use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use rand::prelude::*;
use std::cell::RefCell;

fn main() {
    nannou::app(model).run();
}

// This data structure will be shared across nannou functions.
pub struct Model {
    pd: libpd_rs::Pd,
    output_stream: audio::Stream<()>,
    gravity: f32,
    bubbles: RefCell<Vec<Bubble>>,
    bubble_count: usize,
    time: RefCell<f32>,
}

fn model(app: &App) -> Model {
    // Create window
    app.new_window()
        .size(1280, 640)
        .transparent(true)
        .decorations(true)
        .title("Pd ❤️ Nannou")
        .view(view)
        .build()
        .unwrap();

    let window = app.main_window();
    let win = window.rect();

    let sample_rate = 44100;
    let channels = 2;

    // Initialize the audio host so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    // You may pick another audio device if you wish by its name:

    // use audio::Device;
    // let devices = audio_host.output_devices().unwrap();
    // let device = devices
    //     .into_iter()
    //     .find(|d| d.name().unwrap() == "BlackHole 16ch");

    // Start the stream registering our audio callback.
    let output_stream = audio_host
        .new_output_stream(())
        // Uncomment to pick another audio device.
        // .device(device.unwrap())
        .channels(channels)
        .sample_rate(sample_rate)
        .render(audio_callback)
        .build()
        .unwrap();

    // Listen for console messages from pd
    libpd_rs::functions::receive::on_print(|val| {
        println!("{}", val);
    });

    // This data structure will be shared across nannou functions.
    let mut model = Model {
        // Initialize pd
        pd: libpd_rs::Pd::init_and_configure(0, channels as i32, sample_rate as i32).unwrap(),
        output_stream,
        gravity: 0.8,
        bubbles: RefCell::new(vec![]),
        bubble_count: 13,
        time: RefCell::new(0.),
    };

    let patches_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("with_nannou")
        .join("patches");

    // We're using multiple patches this time, so we need to tell pd where to find them.
    model.pd.add_path_to_search_paths(&patches_dir).unwrap();

    // Load our patches.
    model.pd.open_patch(patches_dir.join("bubbles.pd")).unwrap();

    // Initially pd needs to know how many bubbles we have.
    // Because it will create adequate amount of voices for them.
    libpd_rs::functions::send::send_float_to("bubble_count", model.bubble_count as f32).unwrap();

    // Run pd!
    model.pd.activate_audio(true).unwrap();
    model.output_stream.play().unwrap();

    // Make the bubbles!
    model.make_bubbles(&win);
    model
}

impl Model {
    // Make a single bubble
    fn make_bubble(win: &Rect, id: usize) -> Bubble {
        let mut rng = rand::thread_rng();
        // Random size
        let radius = rng.gen::<f32>() * 150.0;
        // Always falls from the top
        let y_pos = win.h();
        // Do not overflow from the screen
        let candidate_x_pos = rng.gen::<f32>() * win.w() - radius * 2.0;
        let x_pos = if candidate_x_pos <= radius * 2.0 {
            radius * 2.0
        } else {
            candidate_x_pos
        };

        Bubble::new(x_pos, y_pos, radius, rng.gen::<f32>(), id)
    }

    // Make bubbles!
    fn make_bubbles(&mut self, win: &Rect) {
        self.bubbles.borrow_mut().clear();
        for i in 0..self.bubble_count {
            self.bubbles.borrow_mut().push(Self::make_bubble(win, i));
        }
    }

    // Replaces an existing bubble with a new one
    fn replace_bubble(&mut self, index: usize, win: &Rect) {
        if self.bubbles.borrow_mut().get(index).is_some() {
            self.bubbles.borrow_mut()[index] = Self::make_bubble(win, index);
        }
    }
}

// This is where we process audio.
// We hand over all tasks to our pd patch!
fn audio_callback(_: &mut (), buffer: &mut Buffer) {
    let ticks =
        libpd_rs::functions::util::calculate_ticks(buffer.channels() as i32, buffer.len() as i32);
    libpd_rs::functions::process::process_float(ticks, &[], buffer);
}

// This is where we draw repeatedly!
fn view(app: &App, model: &Model, frame: Frame) {
    // Let's poll pd messages here, for every frame.
    libpd_rs::functions::receive::receive_messages_from_pd();

    let background_color = nannou::color::srgb8(238, 108, 77);

    // Draw background
    frame.clear(background_color);
    app.draw().background().color(background_color);

    // Update bubbles!
    for (index, bubble) in model.bubbles.borrow_mut().iter_mut().enumerate() {
        bubble.add_to_frame(app, &frame, index);
        bubble.update(app, model, &frame, index);
    }
}
