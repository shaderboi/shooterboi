use crate::audio::AudioContext;
use crate::database::Database;
use crate::gui::ConrodHandle;
use crate::input_manager::InputManager;
use crate::renderer::Renderer;
use crate::scene::classic_game_scene::ClassicGameScene;
use crate::scene::classic_score_scene::ClassicScoreScene;
use crate::scene::{main_menu_scene::MainMenuScene, Scene, SceneOp, Value};
use crate::window::Window;
use instant::{Duration, Instant};
use std::collections::{HashMap, VecDeque};
use std::ops::Sub;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::Window as WinitWindow;

conrod_winit::v023_conversion_fns!();

pub struct Game {
    scene_stack: VecDeque<Box<dyn Scene>>,
    renderer: Renderer,
    last_time: Instant,
    running_time: Instant,
    window: Window,
    input_manager: InputManager,
    conrod_handle: ConrodHandle,
    audio_context: AudioContext,
    database: Database,
}

impl Game {
    pub fn new(window: WinitWindow) -> Self {
        let mut renderer = pollster::block_on(Renderer::new(&window));
        // self.window.set_is_cursor_grabbed(true);
        let mut conrod_handle = ConrodHandle::new(&mut renderer);
        conrod_handle.get_ui_mut().handle_event(
            convert_event::<()>(
                &Event::WindowEvent {
                    window_id: window.id(),
                    event: WindowEvent::Resized(window.inner_size()),
                },
                &window,
            )
            .unwrap(),
        );
        let mut window = Window::from(window);
        let mut audio_context = AudioContext::new();
        let mut scene_stack = VecDeque::<Box<dyn Scene>>::new();
        let mut first_scene = MainMenuScene::new(&mut renderer, &mut conrod_handle); // ClassicScoreScene::new(&mut renderer, &mut conrod_handle);

        first_scene.init(
            None,
            &mut window,
            &mut renderer,
            &mut conrod_handle,
            &mut audio_context,
        );
        scene_stack.push_back(Box::new(first_scene));
        let mut database = Database::new();
        database.init();
        Self {
            window,
            scene_stack,
            conrod_handle,
            renderer,
            input_manager: InputManager::new(),
            last_time: Instant::now(),
            running_time: Instant::now(),
            audio_context,
            database,
        }
    }

    pub fn update(&mut self, event: &Event<()>, control_flow: &mut ControlFlow) {
        if let Some(event) = convert_event(event, &self.window) {
            self.conrod_handle.get_ui_mut().handle_event(event);
        }
        match event {
            Event::WindowEvent { event, window_id } if *window_id == self.window.id() => {
                self.input_manager.process(event);
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        #[cfg(not(target_arch = "wasm32"))]
                        if self.window.is_cursor_grabbed() {
                            let window_size = self.window.inner_size();
                            let center = nalgebra::Vector2::<f32>::new(
                                window_size.width as f32 / 2.0,
                                window_size.height as f32 / 2.0,
                            );

                            let new_pos =
                                nalgebra::Vector2::<f32>::new(position.x as f32, position.y as f32);

                            self.input_manager.mouse_movement += center - new_pos;

                            self.window
                                .set_cursor_position(PhysicalPosition {
                                    x: center.x,
                                    y: center.y,
                                })
                                .unwrap();
                        }
                    }
                    WindowEvent::Resized(physical_size) => {
                        self.renderer
                            .resize(physical_size, self.window.scale_factor());
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        self.renderer
                            .resize(*new_inner_size, self.window.scale_factor());
                    }
                    _ => {}
                };
            }
            Event::MainEventsCleared => {
                let current_time = Instant::now();
                let delta_time = current_time.duration_since(self.last_time).as_secs_f32();
                self.last_time = current_time;

                // #[cfg(target_arch = "wasm32")]
                if self.window.is_cursor_grabbed() {
                    {
                        let mut dir_diff = nalgebra::Vector2::new(0.0, 0.0);
                        if self.input_manager.is_keyboard_press(&VirtualKeyCode::Left) {
                            dir_diff.x += 400.0 * delta_time;
                        } else if self.input_manager.is_keyboard_press(&VirtualKeyCode::Right) {
                            dir_diff.x -= 400.0 * delta_time;
                        }

                        if self.input_manager.is_keyboard_press(&VirtualKeyCode::Up) {
                            dir_diff.y += 400.0 * delta_time;
                        } else if self.input_manager.is_keyboard_press(&VirtualKeyCode::Down) {
                            dir_diff.y -= 400.0 * delta_time;
                        }

                        self.input_manager.mouse_movement += dir_diff;
                    }
                }

                // const MAX_FRAME_TIME: f32 = 0.2;
                // const FIXED_TIMESTEP: f32 = 1.0 / 20.0;

                let scene_op = self.scene_stack.back_mut().unwrap().update(
                    &mut self.window,
                    &mut self.renderer,
                    &self.input_manager,
                    delta_time,
                    &mut self.conrod_handle,
                    &mut self.audio_context,
                    control_flow,
                );

                match scene_op {
                    SceneOp::None => {}
                    SceneOp::Pop(layer_number, message) => {
                        for _ in 0..layer_number {
                            self.scene_stack.back_mut().unwrap().deinit(
                                &mut self.window,
                                &mut self.renderer,
                                &mut self.conrod_handle,
                                &mut self.audio_context,
                            );
                            self.scene_stack.pop_back();
                        }
                        self.scene_stack.back_mut().unwrap().init(
                            message,
                            &mut self.window,
                            &mut self.renderer,
                            &mut self.conrod_handle,
                            &mut self.audio_context,
                        );
                    }
                    SceneOp::Push(mut new_scene, message) => {
                        if let Some(prev_scene) = self.scene_stack.back_mut() {
                            prev_scene.deinit(
                                &mut self.window,
                                &mut self.renderer,
                                &mut self.conrod_handle,
                                &mut self.audio_context,
                            );
                        }
                        new_scene.init(
                            message,
                            &mut self.window,
                            &mut self.renderer,
                            &mut self.conrod_handle,
                            &mut self.audio_context,
                        );
                        self.scene_stack.push_back(new_scene);
                    }
                    SceneOp::Replace(mut new_scene, message) => {
                        self.scene_stack.back_mut().unwrap().deinit(
                            &mut self.window,
                            &mut self.renderer,
                            &mut self.conrod_handle,
                            &mut self.audio_context,
                        );
                        self.scene_stack.pop_back();
                        new_scene.init(
                            message,
                            &mut self.window,
                            &mut self.renderer,
                            &mut self.conrod_handle,
                            &mut self.audio_context,
                        );
                        self.scene_stack.push_back(new_scene);
                    }
                };

                self.scene_stack.back_mut().unwrap().prerender(
                    &mut self.renderer,
                    &self.input_manager,
                    delta_time,
                    &mut self.conrod_handle,
                    &mut self.audio_context,
                );

                match self.renderer.render(
                    self.running_time.elapsed().as_secs_f32(),
                    &mut self.conrod_handle,
                ) {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => self.renderer.resize(
                        &PhysicalSize {
                            width: self.renderer.surface_and_window_config.surface.width,
                            height: self.renderer.surface_and_window_config.surface.height,
                        },
                        self.window.scale_factor(),
                    ),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                };

                self.input_manager.clear();
                self.audio_context.clear();
            }
            _ => {}
        };
    }
}
