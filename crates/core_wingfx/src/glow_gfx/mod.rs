use core_renderer::Renderer;
use core_simulation::{ControlMessage, Input, Simulation, SimulationExecutor, WindowMsg};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, ContextWrapper};

mod glow_render;
use glow_render::make;

use crate::wingfx::WinGfx;

pub struct OpenGlWindow {
    title: &'static str,
    w: u32,
    h: u32,
}

impl OpenGlWindow {
    pub fn new(title: &'static str, w: u32, h: u32) -> Self {
        Self { title, w, h }
    }

    fn handle_event<T>(event: Event<T>, control_flow: &mut ControlFlow) -> Option<WindowMsg> {
        match event {
            Event::LoopDestroyed => None,
            Event::MainEventsCleared => Some(WindowMsg::RedrawRequested),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => Some(WindowMsg::Resize {
                    w: physical_size.width,
                    h: physical_size.height,
                }),
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    Some(WindowMsg::Shutdown)
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    let pressed = match input.state {
                        glutin::event::ElementState::Pressed => true,
                        glutin::event::ElementState::Released => false,
                    };

                    let mut key_msg = core_simulation::KeyboardMsg::W;

                    if let Some(keycode) = input.virtual_keycode {
                        match keycode {
                            glutin::event::VirtualKeyCode::W => {
                                key_msg = core_simulation::KeyboardMsg::W;
                            }
                            glutin::event::VirtualKeyCode::A => {
                                key_msg = core_simulation::KeyboardMsg::A;
                            }
                            glutin::event::VirtualKeyCode::S => {
                                key_msg = core_simulation::KeyboardMsg::S;
                            }
                            glutin::event::VirtualKeyCode::D => {
                                key_msg = core_simulation::KeyboardMsg::D;
                            }
                            _ => {}
                        }

                        if pressed {
                            return Some(WindowMsg::KeyPress(key_msg));
                        } else {
                            return Some(WindowMsg::KeyRelease(key_msg));
                        }
                    }

                    None
                }
                _ => None,
            },
            Event::RedrawRequested(_) => Some(WindowMsg::RedrawRequested),
            _ => None,
        }
    }
}

impl<Sim, Cfg, Msg> WinGfx<Sim, Cfg, Msg> for OpenGlWindow
where
    Sim: Simulation<Cfg, Msg> + 'static,
    Cfg: 'static,
    Msg: 'static,
{
    fn begin_execution(&mut self, mut executor: SimulationExecutor<Sim, Cfg, Msg>) {
        let el = glutin::event_loop::EventLoop::new();
        let wb = glutin::window::WindowBuilder::new()
            .with_title(self.title)
            .with_inner_size(glutin::dpi::LogicalSize::new(self.w as f32, self.h as f32));

        let windowed_context = glutin::ContextBuilder::new()
            .with_vsync(false)
            .build_windowed(wb, &el)
            .unwrap();

        let windowed_context = unsafe { windowed_context.make_current().unwrap() };

        let mut renderer =
            core_renderer::make_renderer(Box::new(make(self.w, self.h, &windowed_context)));

        let mut last_rendered_frame = 0;
        let mut shutdown = false;

        el.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            let ev = Self::handle_event(event, control_flow);

            let ev: Option<Input<Msg>> = match &ev {
                Some(ev) => {
                    match ev {
                        WindowMsg::RedrawRequested => {
                            renderer.dispatch();
                            windowed_context.swap_buffers().unwrap();
                        }
                        WindowMsg::Shutdown => {
                            shutdown = true;
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowMsg::Resize { w, h } => {
                            let w = *w;
                            let h = *h;
                            windowed_context.resize(glutin::dpi::PhysicalSize::new(w, h));
                            renderer.resize(w, h);
                        }
                        WindowMsg::KeyPress(_) => {}
                        WindowMsg::KeyRelease(_) => {}
                    }

                    Some(Input::WindowMsg(*ev))
                }
                None => None,
            };

            match executor.tick(ev) {
                core_simulation::ControlMessage::Ok => {}
                core_simulation::ControlMessage::ExitSim => {
                    *control_flow = ControlFlow::Exit;
                }
            }

            if executor.last_updated_frame() != last_rendered_frame {
                last_rendered_frame = executor.last_updated_frame();
                renderer.create_render_pass();
                executor.sim_mut().render(&mut renderer);
            }
        });
    }
}
