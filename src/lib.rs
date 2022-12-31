extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use piston::EventLoop;
use rand::Rng;

use glutin_window::GlutinWindow as Window;
use graphics::{Context, Transformed};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

struct ParticleRules {
    red_on_red: f64,
    red_on_green: f64,
    red_on_blue: f64,
    red_on_yellow: f64,
    green_on_red: f64,
    green_on_green: f64,
    green_on_blue: f64,
    green_on_yellow: f64,
    blue_on_red: f64,
    blue_on_green: f64,
    blue_on_blue: f64,
    blue_on_yellow: f64,
    yellow_on_red: f64,
    yellow_on_green: f64,
    yellow_on_blue: f64,
    yellow_on_yellow: f64,
}

impl ParticleRules {
    fn randomise(limit: f64) -> Self {
        ParticleRules {
            red_on_red: rand::thread_rng().gen_range(-limit..limit),
            red_on_green: rand::thread_rng().gen_range(-limit..limit),
            red_on_blue: rand::thread_rng().gen_range(-limit..limit),
            red_on_yellow: rand::thread_rng().gen_range(-limit..limit),
            green_on_red: rand::thread_rng().gen_range(-limit..limit),
            green_on_green: rand::thread_rng().gen_range(-limit..limit),
            green_on_blue: rand::thread_rng().gen_range(-limit..limit),
            green_on_yellow: rand::thread_rng().gen_range(-limit..limit),
            blue_on_red: rand::thread_rng().gen_range(-limit..limit),
            blue_on_green: rand::thread_rng().gen_range(-limit..limit),
            blue_on_blue: rand::thread_rng().gen_range(-limit..limit),
            blue_on_yellow: rand::thread_rng().gen_range(-limit..limit),
            yellow_on_red: rand::thread_rng().gen_range(-limit..limit),
            yellow_on_green: rand::thread_rng().gen_range(-limit..limit),
            yellow_on_blue: rand::thread_rng().gen_range(-limit..limit),
            yellow_on_yellow: rand::thread_rng().gen_range(-limit..limit),
        }
    }
}
#[derive(Copy, Clone, Debug)]
enum ParticleType {
    Red,
    Green,
    Yellow,
    Blue,
}
#[derive(Copy, Clone, Debug)]
struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    fx: f64,
    fy: f64,
    radius: f64,
    color: [f32; 4],
    particle_type: ParticleType,
}

impl Particle {
    fn new(width: f64, height: f64, color: [f32; 4], particle_type: ParticleType) -> Self {
        let rand_x = rand::thread_rng().gen_range(0..width as i32);
        let rand_y = rand::thread_rng().gen_range(0..height as i32);

        Particle {
            x: rand_x as f64,
            y: rand_y as f64,
            vx: 0.0,
            vy: 0.0,
            fx: 0.0,
            fy: 0.0,
            radius: 3.0,
            color,
            particle_type,
        }
    }

    fn render(&self, c: Context, gl: &mut GlGraphics, args: &RenderArgs) {
        let transform = c.transform.trans(-self.radius, -self.radius);
        let rectangle = [self.x, self.y, self.radius * 2.0, self.radius * 2.0];
        graphics::ellipse(self.color, rectangle, transform, gl);
    }

    fn update(&mut self, other: &ParticleInfo, rules: &ParticleRules, width: f64, height: f64) {
        let damper = 0.5;
        let viscosity = 0.95;

        let dx = self.x - other.x;
        let dy = self.y - other.y;

        let distance = ((&dx * &dx) + (&dy * &dy)).sqrt();
        if distance > 0.0 && distance < 80.0 {
            // * Formula for gravitational force assuming m1 and m2 are 1
            let g = match (&self.particle_type, &other.particle_type) {
                (ParticleType::Red, ParticleType::Red) => rules.red_on_red,
                (ParticleType::Red, ParticleType::Green) => rules.red_on_green,
                (ParticleType::Red, ParticleType::Yellow) => rules.red_on_yellow,
                (ParticleType::Red, ParticleType::Blue) => rules.red_on_blue,
                (ParticleType::Green, ParticleType::Red) => rules.green_on_red,
                (ParticleType::Green, ParticleType::Green) => rules.green_on_green,
                (ParticleType::Green, ParticleType::Yellow) => rules.green_on_yellow,
                (ParticleType::Green, ParticleType::Blue) => rules.green_on_blue,
                (ParticleType::Yellow, ParticleType::Red) => rules.yellow_on_red,
                (ParticleType::Yellow, ParticleType::Green) => rules.yellow_on_green,
                (ParticleType::Yellow, ParticleType::Yellow) => rules.yellow_on_yellow,
                (ParticleType::Yellow, ParticleType::Blue) => rules.yellow_on_blue,
                (ParticleType::Blue, ParticleType::Red) => rules.blue_on_red,
                (ParticleType::Blue, ParticleType::Green) => rules.blue_on_green,
                (ParticleType::Blue, ParticleType::Yellow) => rules.blue_on_yellow,
                (ParticleType::Blue, ParticleType::Blue) => rules.blue_on_blue,
            };
            let force = g * 1.0 / distance;

            self.fx = force * dx;
            self.fy = force * dy;

            self.vx += self.fx * damper;
            self.vy += self.fy * damper;

            self.vx *= viscosity;
            self.vy *= viscosity;

            self.x += self.vx;
            self.y += self.vy;

            if self.x < 0.0 || self.x > width {
                self.vx *= -1.0;
            }
            if self.y < 0.0 || self.y > height {
                self.vy *= -1.0;
            }
        }
    }
}

struct Simulator {
    gl: GlGraphics,
    particles: Vec<Particle>,
    num_red: u32,
    num_blue: u32,
    num_green: u32,
    num_yellow: u32,
}

impl Simulator {
    fn new(
        width: f64,
        height: f64,
        gl: GlGraphics,
        num_red: u32,
        num_green: u32,
        num_blue: u32,
        num_yellow: u32,
    ) -> Self {
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
        let mut particles = vec![];

        for _ in 0..num_red {
            particles.push(Particle::new(width, height, RED, ParticleType::Red));
        }
        for _ in 0..num_green {
            particles.push(Particle::new(width, height, GREEN, ParticleType::Green));
        }
        for _ in 0..num_blue {
            particles.push(Particle::new(width, height, BLUE, ParticleType::Blue));
        }
        for _ in 0..num_yellow {
            particles.push(Particle::new(width, height, YELLOW, ParticleType::Yellow));
        }
        Simulator {
            gl,
            particles,
            num_red,
            num_blue,
            num_green,
            num_yellow,
        }
    }
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);
            for particle in self.particles.iter() {
                particle.render(c, gl, args);
            }
        });
    }


    fn update(&mut self, rules: &ParticleRules, width: f64, height: f64) {
        let length = self.particles.len();

        for i in 0..length {
            let p1 = ParticleInfo {
                x: self.particles[i].x,
                y: self.particles[i].y,
                particle_type: self.particles[i].particle_type,
            };
            for j in 0..length {
                let p2 = &mut self.particles[j];

                p2.update(&p1, rules, width, height);
            }
        }
    }
}

struct ParticleInfo {
    x: f64,
    y: f64,
    particle_type: ParticleType,
}

pub fn run() {
    let opengl = OpenGL::V3_2;

    const WIDTH: f64 = 1000.0;
    const HEIGHT: f64 = 600.0;

    const NUM_RED: u32 = 300;
    const NUM_GREEN: u32 = 300;
    const NUM_BLUE: u32 = 300;
    const NUM_YELLOW: u32 = 300;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("Particle life", [WIDTH, HEIGHT])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let rules: ParticleRules = ParticleRules::randomise(1.0);

    let mut sim = Simulator::new(
        WIDTH,
        HEIGHT,
        GlGraphics::new(opengl),
        NUM_RED,
        NUM_GREEN,
        NUM_BLUE,
        NUM_YELLOW,
    );

    let mut events = Events::new(EventSettings::new()).ups(30);
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            sim.render(&args);
        }
        if let Some(args) = e.update_args() {
            sim.update(&rules, WIDTH, HEIGHT);
        }
    }
}
