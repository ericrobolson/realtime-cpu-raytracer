use std::marker::PhantomData;

use core_data_structures::queue::Queue;
pub use core_renderer::{RenderBuilder, RenderCommand};
pub use core_timing::Duration;
use core_timing::{hz_to_duration, Stopwatch};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ValkErr {}

/// Messages a simulation may pass back to the engine
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ControlMessage {
    Ok,
    ExitSim,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Input<UserMsg> {
    UserMsg(UserMsg),
    WindowMsg(WindowMsg),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WindowMsg {
    Shutdown,
    RedrawRequested,
    Resize { w: u32, h: u32 },
    KeyPress(KeyboardMsg),
    KeyRelease(KeyboardMsg),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum KeyboardMsg {
    W,
    A,
    S,
    D,
}

/// Common functionality a simulation must implement.
pub trait Simulation<Cfg, Msg> {
    /// Creates a new simulation.
    fn new(config: Cfg) -> Self;

    /// A single 'tick' for an application.
    fn tick(&mut self, frame: u64, delta_t: Duration, messages: &[Input<Msg>]) -> ControlMessage;

    /// Queues up a render pass.
    fn render(&mut self, renderer: &mut impl RenderBuilder);
}

/// Executor for simulation. Handles timestepping.
pub struct SimulationExecutor<Sim, Cfg, Msg>
where
    Sim: Simulation<Cfg, Msg>,
{
    use_fixed_timestep: bool,
    time_keeper: Timekeeper,
    sim: Sim,
    engine_queue: Queue<Input<Msg>>,
    cfg_phantom: PhantomData<Cfg>,
    frame: u64,
}

impl<Sim, Cfg, Msg> SimulationExecutor<Sim, Cfg, Msg>
where
    Sim: Simulation<Cfg, Msg>,
{
    /// Creates a new SimulationExecutor
    pub fn new(
        max_engine_msgs: usize,
        sim_hz: Option<u32>,
        use_fixed_timestep: bool,
        config: Cfg,
    ) -> Self {
        let sim_hz = match sim_hz {
            Some(hz) => hz.max(1),
            None => 0,
        };

        let time_keeper = Timekeeper {
            tick_duration: hz_to_duration(sim_hz),
            accumulated_time: Duration::from_secs(0),
            simulation_stopwatch: Stopwatch::new(),
        };

        let sim = Sim::new(config);

        Self {
            use_fixed_timestep,
            time_keeper,
            sim,
            engine_queue: Queue::new(max_engine_msgs),
            cfg_phantom: PhantomData,
            frame: 0,
        }
    }

    pub fn sim(&self) -> &Sim {
        &self.sim
    }

    pub fn sim_mut(&mut self) -> &mut Sim {
        &mut self.sim
    }

    /// Returns the last updated frame. Will wrap to 0 when it reaches the max value.
    pub fn last_updated_frame(&self) -> u64 {
        self.frame
    }

    /// Passes in the input message and attempts to execute.
    pub fn tick(&mut self, input: Option<Input<Msg>>) -> ControlMessage {
        let mut control_msg = ControlMessage::Ok;

        // Queue up any messages
        if let Some(input) = input {
            self.engine_queue.push(input);
        }

        // If we're using a fixed time step, see if it should be executed.
        if self.use_fixed_timestep {
            // Increase accumulated time + tick if necessary
            // Based on https://gafferongames.com/post/fix_your_timestep/ to divorce rendering + simulations

            self.time_keeper.accumulated_time += self.time_keeper.simulation_stopwatch.elapsed();

            // In the event that the loop gets in a spiral of death where the sim can't keep up,
            // clamp it to a set number of ticks per frame to prevent spiraling downward.
            const MAX_TICKS_PER_FRAME: u8 = 10;
            let mut times_ticked = 0;

            // Tick the simulation until it has caught up
            while self.time_keeper.accumulated_time > self.time_keeper.tick_duration {
                self.frame = self.frame.wrapping_add(1);
                self.time_keeper.accumulated_time -= self.time_keeper.tick_duration;
                times_ticked += 1;

                control_msg = self.sim.tick(
                    self.frame,
                    self.time_keeper.tick_duration,
                    &self.engine_queue.items(),
                );
                self.engine_queue.clear();

                // Break out if the sim is taking too long. Or it should shut down.
                // This way it keeps processing and doesn't get stuck in a horrendous loop. It'll slow the game down
                // to a crawl, but at least it isn't preventing people from playing.
                if times_ticked >= MAX_TICKS_PER_FRAME || control_msg == ControlMessage::ExitSim {
                    break;
                }
            }
        }
        // Otherwise execute it
        else {
            self.frame = self.frame.wrapping_add(1);
            let delta_t = self.time_keeper.simulation_stopwatch.elapsed();
            control_msg = self
                .sim
                .tick(self.frame, delta_t, &self.engine_queue.items());
            self.engine_queue.clear();
        }

        // Return the control message
        control_msg
    }
}

/// Time tracking record manager
struct Timekeeper {
    tick_duration: Duration,
    accumulated_time: Duration,
    simulation_stopwatch: Stopwatch,
}
