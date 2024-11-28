use crate::Model;
use nannou::{App, Frame};
use rand::prelude::*;

/// A struct which holds the parameters for a single voice attached to that bubble in pd.
#[derive(Debug, Clone)]
pub struct BubbleMessage {
    instance_number: usize,
    gain: f32,
    osc_freq: f32,
    filter_freq: f32,
    filter_q: f32,
    attack_dest: f32,
    attack_time: f32,
    release_dest: f32,
    release_time: f32,
    reverb_size: f32,
    reverb_wet: f32,
    reverb_freeze: f32,
    reverb_bypass: f32,
}

// We're going to make this message an atom list to send to pure data.
// It would be convenient to implement Iterator for it.
pub struct BubbleMessageIterator<'a> {
    bubble_message: &'a BubbleMessage,
    index: usize,
}

impl Default for BubbleMessage {
    fn default() -> Self {
        BubbleMessage {
            instance_number: 0,
            gain: 1.0,
            osc_freq: 120.0,
            filter_freq: 2500.0,
            filter_q: 2.0,
            attack_dest: 1.0,
            attack_time: 1.0,
            release_dest: 0.0,
            release_time: 1.0,
            reverb_size: 0.8,
            reverb_wet: 0.1,
            reverb_freeze: 0.0,
            reverb_bypass: 0.0,
        }
    }
}

impl<'a> IntoIterator for &'a BubbleMessage {
    type Item = f32;
    type IntoIter = BubbleMessageIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BubbleMessageIterator {
            bubble_message: self,
            index: 0,
        }
    }
}

impl<'a> Iterator for BubbleMessageIterator<'a> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let result = match self.index {
            0 => self.bubble_message.instance_number as f32,
            1 => self.bubble_message.gain,
            2 => self.bubble_message.osc_freq,
            3 => self.bubble_message.filter_freq,
            4 => self.bubble_message.filter_q,
            5 => self.bubble_message.attack_dest,
            6 => self.bubble_message.attack_time,
            7 => self.bubble_message.release_dest,
            8 => self.bubble_message.release_time,
            9 => self.bubble_message.reverb_size,
            10 => self.bubble_message.reverb_wet,
            11 => self.bubble_message.reverb_freeze,
            12 => self.bubble_message.reverb_bypass,
            _ => return None,
        };
        self.index += 1;
        Some(result)
    }
}

/// Properties of a bubble.
#[derive(Debug, Default)]
pub struct BubbleProperties {
    /// Current x position of the bubble.
    x: f32,
    /// Current y position of the bubble.
    y: f32,
    /// Delta x position of the bubble.
    dx: f32,
    /// Delta y position of the bubble.
    dy: f32,
    /// Radius of the bubble.
    r: f32,
    /// Alpha amount (opacity) of the bubble.
    alpha: f32,
    /// Friction amount of the bubble.
    friction: f32,
    /// Color of the bubble.
    color: (f32, f32, f32),
}

#[derive(Debug, Default)]
pub struct BubbleState {
    message: BubbleMessage,
}

/// A bubble jumping in the app!
#[derive(Debug, Default)]
pub struct Bubble {
    pub properties: BubbleProperties,
    pub state: BubbleState,
}

impl Bubble {
    pub fn new(x: f32, y: f32, r: f32, alpha: f32, id: usize) -> Self {
        Self {
            properties: BubbleProperties {
                x,
                y,
                r,
                alpha,
                dx: 0.0,
                dy: 0.0,
                // We'd like to slightly change the friction amount for each bubble depending on their size.
                friction: Self::scale(r, 0.0, 150.0, 0.95, 0.85),
                color: (0.537, 0.811, 0.941),
            },
            state: BubbleState {
                message: BubbleMessage {
                    instance_number: id,
                    ..Default::default()
                },
            },
        }
    }

    /// Transforms the voice message of the bubble to send to pure data.
    pub fn pack_message(&self) -> Vec<libpd_rs::atom::Atom> {
        self.state
            .message
            .into_iter()
            .map(std::convert::Into::into)
            .collect()
    }

    pub fn gain(&mut self, value: f32) {
        self.state.message.gain = value
    }
    pub fn osc_freq(&mut self, value: f32) {
        self.state.message.osc_freq = value
    }
    pub fn filter_freq(&mut self, value: f32) {
        self.state.message.filter_freq = value
    }
    pub fn filter_q(&mut self, value: f32) {
        self.state.message.filter_q = value
    }
    pub fn attack_dest(&mut self, value: f32) {
        self.state.message.attack_dest = value
    }
    pub fn attack_time(&mut self, value: f32) {
        self.state.message.attack_time = value
    }
    pub fn release_dest(&mut self, value: f32) {
        self.state.message.release_dest = value
    }
    pub fn release_time(&mut self, value: f32) {
        self.state.message.release_time = value
    }
    pub fn reverb_size(&mut self, value: f32) {
        self.state.message.reverb_size = value
    }
    pub fn reverb_wet(&mut self, value: f32) {
        self.state.message.reverb_wet = value
    }
    pub fn reverb_freeze(&mut self, value: f32) {
        self.state.message.reverb_freeze = value
    }
    pub fn reverb_bypass(&mut self, value: f32) {
        self.state.message.reverb_bypass = value
    }
    pub fn instance_number(&mut self, value: usize) {
        self.state.message.instance_number = value
    }

    /// General scale function.
    pub fn scale(input: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
        let input_range = in_max - in_min;
        let output_range = out_max - out_min;
        let input_scaled = (input - in_min) / input_range;
        input_scaled * output_range + out_min
    }

    /// This function is called every frame.
    pub fn update(&mut self, app: &App, model: &Model, _: &Frame, index: usize) {
        let mut rng = rand::thread_rng();
        let window = app.main_window();
        let win = window.rect();

        // We'll target the right voice in pd for the bubble.
        self.instance_number(index);

        let distance_to_floor = self.properties.y + self.properties.r + self.properties.dy;

        // We'd like less loud collisions when bubbles are closer to the floor.
        self.gain(Self::scale(distance_to_floor, 0.0, win.h(), 0.0, 1.0));

        // Collision with the floor!
        if distance_to_floor < self.properties.r * 2.0 {
            model.pd.set_as_current();
            // On collision we tell the right voice to play with the right parameters in pd.
            libpd_rs::functions::send::send_list_to("bubble_collision", &self.pack_message())
                .unwrap();

            // Physics
            self.properties.dy = -self.properties.dy;
            self.properties.dy *= self.properties.friction;
            self.properties.dx *= self.properties.friction;

            // Make the bubbles smaller when they collide, every time they collide.
            if self.properties.r - 5.0 <= 0.0 {
                self.properties.r = 0.2;
            } else {
                self.properties.r -= 5.0;
            }

            // Dependent to radius
            self.properties.friction = Self::scale(self.properties.r, 0.0, 150.0, 0.95, 0.85);
            self.reverb_size(Self::scale(self.properties.r, 0.0, 150.0, 0.1, 0.5));
            self.filter_freq(Self::scale(self.properties.r, 0.0, 150.0, 3000.0, 160.0));
            self.gain(Self::scale(self.properties.r, 20.0, 60.0, 0.2, 1.0));
            self.filter_q(Self::scale(self.properties.r, 50.0, 150.0, 2.0, 3.5));

            // Make bubbles cyan when closer to the floor
            if self.properties.dy <= 5.0 {
                self.properties.color = (0., 1., 1.);
            }

            // We don't want the bubbles to get stuck in the floor.
            // It is time that a bubble disappears.
            if self.properties.dy <= 0.05 {
                // When a bubble disappears, it should be re-spawned from the top and start falling.

                // Color
                // self.properties.color = (1., 1., 1.);
                // Random size
                self.properties.r = rng.gen::<f32>() * 150.0;
                // Falls from the top
                self.properties.y = win.h();

                // Stays in the bounds of the screen
                let candidate_x_pos = rng.gen::<f32>() * win.w() - self.properties.r * 2.0;
                let x_pos = if candidate_x_pos <= self.properties.r * 2.0 {
                    self.properties.r * 2.0
                } else {
                    candidate_x_pos
                };
                self.properties.x = x_pos;
                // With random opacity
                self.properties.alpha = rng.gen::<f32>();

                // Dependent on size
                self.properties.friction = Self::scale(self.properties.r, 0.0, 150.0, 0.95, 0.85);
                self.reverb_size(Self::scale(self.properties.r, 0.0, 150.0, 0.1, 0.5));
                self.filter_freq(Self::scale(self.properties.r, 0.0, 150.0, 3000.0, 160.0));
                self.gain(Self::scale(self.properties.r, 20.0, 60.0, 0.2, 1.0));
                self.filter_q(Self::scale(self.properties.r, 50.0, 150.0, 2.0, 3.5));

                // Dependent on opacity
                self.osc_freq(Self::scale(self.properties.alpha, 1.0, 0.0, 140.0, 1440.0));
                self.reverb_wet(Self::scale(self.properties.alpha, 1.0, 0.0, 0.01, 0.4));
                if *model.time.borrow_mut() > 1800. {
                    // Tones
                    self.release_time(
                        (Self::scale(
                            self.properties.alpha,
                            1.0,
                            0.0,
                            *model.time.borrow() / 5. + 1.,
                            *model.time.borrow() / 50. + 1.,
                        ) * model.time.borrow().sin())
                        .abs()
                            + 1.0,
                    );
                    if *model.time.borrow_mut() > 3600. {
                        *model.time.borrow_mut() = 0.;
                    }
                } else {
                    // Clicks
                    self.release_time(Self::scale(self.properties.alpha, 1.0, 0.0, 2.0, 1.0));
                }
            }
        } else {
            // Physics
            self.properties.dy -= model.gravity
        }

        // Physics
        self.properties.y += self.properties.dy;

        // Advance time
        *model.time.borrow_mut() += 0.1;
    }

    /// We draw here
    pub fn add_to_frame(&self, app: &App, frame: &Frame, _index: usize) {
        let draw = app.draw();
        let window = app.main_window();
        let win = window.rect();

        let x = self.properties.x - win.w() * 0.5;
        let y = self.properties.y - win.h() * 0.5;

        let (r, g, b) = self.properties.color;
        draw.ellipse()
            .rgba(r, g, b, self.properties.alpha)
            .w(self.properties.r * 2.0)
            .h(self.properties.r * 2.0)
            .x_y(x, y);
        draw.to_frame(app, frame).unwrap();
    }
}
