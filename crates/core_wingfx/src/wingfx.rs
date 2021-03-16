use crate::glow_gfx::OpenGlWindow;
use core_simulation::{Simulation, SimulationExecutor};

/// Trait for WindowGfx functionality.
pub trait WinGfx<Sim, Cfg, Msg>
where
    Sim: Simulation<Cfg, Msg> + 'static,
    Cfg: 'static,
    Msg: 'static,
{
    /// Begins execution of the application.
    fn begin_execution(&mut self, executor: SimulationExecutor<Sim, Cfg, Msg>);
}

pub fn build<Sim, Cfg, Msg>(title: &'static str, w: u32, h: u32) -> impl WinGfx<Sim, Cfg, Msg>
where
    Sim: Simulation<Cfg, Msg> + 'static,
    Cfg: 'static,
    Msg: 'static,
{
    OpenGlWindow::new(title, w, h)
}
