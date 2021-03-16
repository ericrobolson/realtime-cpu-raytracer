#[macro_use]
mod profiling;

mod core_raytracer;
mod renderer;

use core_raytracer::Raytracer;
use core_renderer::RenderBuilder;
use core_simulation::{Simulation, SimulationExecutor};
use core_time::{duration_from_hz, Duration, Timer};
use core_wingfx::WinGfx;

use renderer::{BloodRenderer, Size};

const PERF_DUMP_OCCURENCE: u64 = 100;

fn main() {
    let max_engine_msgs = 256;
    let sim_hz = Some(60);
    let use_fixed_timestep = true;

    let window_width = 1920;
    let window_height = 1080;
    let initial_render_scalar = 120; // This gets a 16:9 render ratio
    let initial_render_scalar = 120;

    let cfg = Cfg {
        initial_render_scalar,
        window_width,
        window_height,
    };

    let executor: SimulationExecutor<Sim, Cfg, Msg> =
        SimulationExecutor::new(max_engine_msgs, sim_hz, use_fixed_timestep, cfg);

    let mut win_gfx = core_wingfx::build("ASCII - RayTracer", window_width, window_height);

    win_gfx.begin_execution(executor);
}

struct Sim<'a> {
    y: f32,
    x: f32,
    /// If set, scales the resolution until it reaches the target duration
    dynamic_scaling_duration: Option<Duration>,

    window_size: Size,
    frame: u64,
    render_scalar: u32,
    renderer: BloodRenderer<'a>,
    raytracer: Raytracer,
}

struct Cfg {
    initial_render_scalar: u32,
    window_width: u32,
    window_height: u32,
}
enum Msg {}

//asdf
impl<'a> Simulation<Cfg, Msg> for Sim<'a> {
    fn new(config: Cfg) -> Self {
        // Rest of program
        let aa_samples = 0;
        let debug_normals = false;
        let post_process_aa = false;
        let save_renders = false;
        let primary_ray_strength = 5;

        let dynamic_scaling_duration = Some(duration_from_hz(90));

        // Terminal renderer
        let font = "res/unifont-13.0.06.ttf";
        let font_size = 32;

        let renderer = renderer::build(
            config.window_width,
            config.window_height,
            config.initial_render_scalar,
            save_renders,
            font,
            font_size,
        );
        let raytracer = core_raytracer::build(
            renderer.size(),
            aa_samples,
            post_process_aa,
            primary_ray_strength,
            debug_normals,
        );

        let mut y = 0.1;
        let x = -3.;
        Self {
            frame: 0,
            dynamic_scaling_duration,
            window_size: Size {
                width: config.window_width,
                height: config.window_height,
            },
            y,
            x,
            render_scalar: config.initial_render_scalar,
            raytracer,
            renderer,
        }
    }

    fn tick(
        &mut self,
        frame: u64,
        delta_t: core_simulation::Duration,
        messages: &[core_simulation::Input<Msg>],
    ) -> core_simulation::ControlMessage {
        perf!("simulation - tick");
        self.frame = frame;
        self.y += 0.01;
        self.x += 0.01;
        let eye = (self.x, self.y, 1.);
        let target = (0., 0., 0.);

        for msg in messages {
            match msg {
                core_simulation::Input::UserMsg(_) => {}
                core_simulation::Input::WindowMsg(win_msg) => match win_msg {
                    core_simulation::WindowMsg::Shutdown => {}
                    core_simulation::WindowMsg::RedrawRequested => {}
                    core_simulation::WindowMsg::Resize { w, h } => {
                        self.renderer.resize(*w, *h, self.render_scalar);
                        self.raytracer.resize(self.renderer.size());
                    }
                    core_simulation::WindowMsg::KeyPress(_) => {}
                    core_simulation::WindowMsg::KeyRelease(_) => {}
                },
            }
        }

        self.raytracer.look_at(eye, target, None);

        // Write the perf metrics every 100 frames
        #[cfg(feature = "profiling")]
        {
            if frame % PERF_DUMP_OCCURENCE == 0 {
                match profiling::PROFILE_MANAGER.lock() {
                    Ok(mut mgr) => {
                        mgr.flush();
                    }
                    Err(_) => {}
                }
            }
        }

        core_simulation::ControlMessage::Ok
    }

    fn render(&mut self, renderer: &mut impl RenderBuilder) {
        let timer = Timer::new();

        // TODO: timers should probably be averaged.

        self.raytracer.execute_render(self.renderer.queue());
        self.renderer.draw(renderer);

        match self.dynamic_scaling_duration {
            Some(duration) => {
                let mut modified_scaling = false;

                // Scale it 'down'
                if timer.elapsed() > duration {
                    if self.render_scalar < MAX_RENDER_SCALAR {
                        self.render_scalar += 1;
                        modified_scaling = true;
                    }
                }
                // Scale it 'up'
                else if timer.elapsed() < duration {
                    if self.render_scalar > MIN_RENDER_SCALAR {
                        self.render_scalar -= 1;
                        modified_scaling = true;
                    }
                }

                if modified_scaling {
                    self.renderer.resize(
                        self.window_size.width,
                        self.window_size.height,
                        self.render_scalar,
                    );
                    self.raytracer.resize(self.renderer.size());
                }
            }
            None => {}
        }
    }
}

const MAX_RENDER_SCALAR: u32 = 120;
const MIN_RENDER_SCALAR: u32 = 12;
