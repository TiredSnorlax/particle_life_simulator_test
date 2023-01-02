extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use piston::{Button, ButtonEvent, EventLoop, Key, PressEvent};
use rand::Rng;

use std::process;
use std::slice::Iter;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use glutin_window::GlutinWindow as Window;
use graphics::{Context, Transformed};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

#[derive(Debug)]
struct ParticleRules {
    red_on_red: (f64, f64),
    red_on_green: (f64, f64),
    red_on_blue: (f64, f64),
    red_on_yellow: (f64, f64),
    green_on_red: (f64, f64),
    green_on_green: (f64, f64),
    green_on_blue: (f64, f64),
    green_on_yellow: (f64, f64),
    blue_on_red: (f64, f64),
    blue_on_green: (f64, f64),
    blue_on_blue: (f64, f64),
    blue_on_yellow: (f64, f64),
    yellow_on_red: (f64, f64),
    yellow_on_green: (f64, f64),
    yellow_on_blue: (f64, f64),
    yellow_on_yellow: (f64, f64),
}

impl ParticleRules {
    fn randomise() -> Self {
        use rand::thread_rng;
        let strength = 1.0;
        let radius_lower = 60.0;
        let radius_higher = 125.0;
        let rules = ParticleRules {
            red_on_red: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            red_on_green: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            red_on_blue: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            red_on_yellow: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            green_on_red: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            green_on_green: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            green_on_blue: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            green_on_yellow: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            blue_on_red: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            blue_on_green: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            blue_on_blue: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            blue_on_yellow: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            yellow_on_red: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            yellow_on_green: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            yellow_on_blue: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
            yellow_on_yellow: (
                thread_rng().gen_range(-strength..strength),
                thread_rng().gen_range(radius_lower..radius_higher),
            ),
        };
        println!("{:?}", rules);
        rules
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
    radius: f64,
    color: [f32; 4],
}

impl Particle {
    fn new(width: f64, height: f64, color: [f32; 4]) -> Self {
        let rand_x = rand::thread_rng().gen_range(0..width as i32);
        let rand_y = rand::thread_rng().gen_range(0..height as i32);

        Particle {
            x: rand_x as f64,
            y: rand_y as f64,
            vx: 0.0,
            vy: 0.0,
            radius: 1.0,
            color,
        }
    }

    fn render(&self, c: Context, gl: &mut GlGraphics, args: &RenderArgs) {
        let transform = c.transform.trans(-self.radius, -self.radius);
        let rectangle = [self.x, self.y, self.radius * 2.0, self.radius * 2.0];
        graphics::rectangle(self.color, rectangle, transform, gl);
    }
}

struct Simulator {
    gl: GlGraphics,
}

impl Simulator {
    fn new(gl: GlGraphics) -> Self {
        Simulator { gl }
    }

    fn new_groups(
        num_red: u32,
        num_blue: u32,
        num_green: u32,
        num_yellow: u32,
        width: f64,
        height: f64,
    ) -> (
        Arc<Mutex<Vec<Particle>>>,
        Arc<Mutex<Vec<Particle>>>,
        Arc<Mutex<Vec<Particle>>>,
        Arc<Mutex<Vec<Particle>>>,
    ) {
        let mut particles_red = vec![];
        let mut particles_blue = vec![];
        let mut particles_green = vec![];
        let mut particles_yellow = vec![];
        for i in 0..num_red {
            particles_red.push(Particle::new(width, height, [1.0, 0.0, 0.0, 1.0]))
        }
        for i in 0..num_blue {
            particles_blue.push(Particle::new(width, height, [0.5, 0.5, 1.0, 1.0]))
        }
        for i in 0..num_green {
            particles_green.push(Particle::new(width, height, [0.0, 1.0, 0.0, 1.0]))
        }
        for i in 0..num_yellow {
            particles_yellow.push(Particle::new(width, height, [1.0, 1.0, 0.0, 1.0]))
        }

        (
            Arc::new(Mutex::new(particles_red)),
            Arc::new(Mutex::new(particles_blue)),
            Arc::new(Mutex::new(particles_green)),
            Arc::new(Mutex::new(particles_yellow)),
        )
    }

    fn render(
        &mut self,
        args: &RenderArgs,
        red_grp: &Arc<Mutex<Vec<Particle>>>,
        blue_grp: &Arc<Mutex<Vec<Particle>>>,
        green_grp: &Arc<Mutex<Vec<Particle>>>,
        yellow_grp: &Arc<Mutex<Vec<Particle>>>,
    ) {
        use graphics::*;
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);
            for particle in (*red_grp.lock().unwrap()).iter() {
                particle.render(c, gl, args);
            }
            for particle in (*blue_grp.lock().unwrap()).iter() {
                particle.render(c, gl, args);
            }
            for particle in (*green_grp.lock().unwrap()).iter() {
                particle.render(c, gl, args);
            }
            for particle in (*yellow_grp.lock().unwrap()).iter() {
                particle.render(c, gl, args);
            }
        });
    }

    fn rules(
        &self,
        grp1: &Arc<Mutex<Vec<Particle>>>,
        grp2: Vec<Particle>,
        rules: (f64, f64),
        width: f64,
        height: f64,
        handles: &mut Vec<JoinHandle<()>>,
        now: Instant,
    ) {
        let grp1 = Arc::clone(grp1);

        let viscosity = 0.7;
        // let delta = now.elapsed().as_micros() as f64 / 1000.0;

        let handle = thread::spawn(move || {
            let mut grp1 = grp1.lock().unwrap_or_else(|err| {
                println!("{:?}", err);
                process::exit(1)
            });

            for particle in grp1.iter_mut() {
                let mut fx = 0.0;
                let mut fy = 0.0;

                for other in grp2.iter() {
                    let dx = particle.x - other.x;
                    let dy = particle.y - other.y;

                    let distance = ((&dx * &dx) + (&dy * &dy)).sqrt();
                    if distance > 0.0 && distance < rules.1 {
                        // * Formula for gravitational force assuming m1 and m2 are 1
                        let force = 1.0 / distance;

                        fx += force * dx;
                        fy += force * dy;
                    }
                }

                if particle.x < 0.0 {
                    particle.vx += 10.0;
                } else if particle.x > width {
                    particle.vx += -10.0;
                }
                if particle.y < 0.0 {
                    particle.vy += 10.0;
                } else if particle.y > height {
                    particle.vy += -10.0;
                }

                particle.vx += fx * rules.0;
                particle.vy += fy * rules.0;

                particle.vx *= 1.0 - viscosity;
                particle.vy *= 1.0 - viscosity;

                particle.x += particle.vx;
                particle.y += particle.vy;
            }
        });
        handles.push(handle);
    }
}

struct ParticleInfo {
    x: f64,
    y: f64,
    particle_type: ParticleType,
}

pub fn run() {
    let opengl = OpenGL::V3_2;

    const WIDTH: f64 = 1200.0;
    const HEIGHT: f64 = 800.0;

    const NUM_RED: u32 = 700;
    const NUM_GREEN: u32 = 700;
    const NUM_BLUE: u32 = 700;
    const NUM_YELLOW: u32 = 700;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("Particle life", [WIDTH, HEIGHT])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut rules: ParticleRules = ParticleRules::randomise();

    let mut sim = Simulator::new(GlGraphics::new(opengl));

    let (mut red_grp, mut blue_grp, mut green_grp, mut yellow_grp) =
        Simulator::new_groups(NUM_RED, NUM_BLUE, NUM_GREEN, NUM_YELLOW, WIDTH, HEIGHT);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        let timer = Instant::now();
        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::Space {
                rules = ParticleRules::randomise();
                (red_grp, blue_grp, green_grp, yellow_grp) =
                    Simulator::new_groups(NUM_RED, NUM_BLUE, NUM_GREEN, NUM_YELLOW, WIDTH, HEIGHT);
            }

            println!("Pressed keyboard key '{:?}'", key);
        };

        if let Some(args) = e.render_args() {
            sim.render(&args, &red_grp, &blue_grp, &green_grp, &yellow_grp);
        }
        if let Some(args) = e.update_args() {
            let now = Instant::now();
            let mut handles: Vec<JoinHandle<()>> = vec![];

            let red_clone = red_grp.lock().unwrap().clone();

            sim.rules(
                &red_grp,
                red_clone.clone(),
                rules.red_on_red,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            sim.rules(
                &blue_grp,
                red_clone.clone(),
                rules.blue_on_red,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            sim.rules(
                &green_grp,
                red_clone.clone(),
                rules.green_on_red,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            sim.rules(
                &yellow_grp,
                red_clone.clone(),
                rules.yellow_on_red,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            let blue_clone = blue_grp.lock().unwrap().clone();

            sim.rules(
                &blue_grp,
                blue_clone.clone(),
                rules.blue_on_blue,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            sim.rules(
                &red_grp,
                blue_clone.clone(),
                rules.red_on_blue,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            sim.rules(
                &yellow_grp,
                blue_clone.clone(),
                rules.yellow_on_blue,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            sim.rules(
                &green_grp,
                blue_clone.clone(),
                rules.green_on_blue,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            let green_clone = green_grp.lock().unwrap().clone();
            sim.rules(
                &green_grp,
                green_clone.clone(),
                rules.green_on_green,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            sim.rules(
                &red_grp,
                green_clone.clone(),
                rules.red_on_green,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            sim.rules(
                &blue_grp,
                green_clone.clone(),
                rules.blue_on_green,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            sim.rules(
                &yellow_grp,
                green_clone.clone(),
                rules.yellow_on_green,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            let yellow_clone = yellow_grp.lock().unwrap().clone();
            sim.rules(
                &blue_grp,
                yellow_clone.clone(),
                rules.blue_on_yellow,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            sim.rules(
                &red_grp,
                yellow_clone.clone(),
                rules.red_on_yellow,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            sim.rules(
                &yellow_grp,
                yellow_clone.clone(),
                rules.yellow_on_yellow,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            sim.rules(
                &green_grp,
                yellow_clone.clone(),
                rules.green_on_yellow,
                WIDTH,
                HEIGHT,
                &mut handles,
                now,
            );

            // for handle in handles {
            //     handle.join().unwrap_or_else(|err| {
            //         println!("{:?}", err);
            //         process::exit(1);
            //     });
            // }
            // println!("{:?}", now.elapsed());
        }
        println!("{:?}", timer.elapsed());
    }
}
