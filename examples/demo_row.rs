use kirei::gui::core::{FocusManager, Rect, Widget, WidgetContext, WidgetStateStorage};
use kirei::gui::renderer::Renderer;
use kirei::gui::theme::Theme;
use kirei::gui::widgets::{Align, Button, Column, Label, Panel, Row, ScrollView};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

struct InputState {
    mouse_pos: glam::Vec2,
    mouse_pressed: bool,
}

impl InputState {
    fn default() -> Self {
        Self {
            mouse_pos: glam::Vec2::ZERO,
            mouse_pressed: false,
        }
    }
}

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    renderer: Renderer,
    root_widget: Box<dyn Widget>,
    input_state: InputState,
    theme: Theme,
    window: Arc<Window>,
    widget_state: WidgetStateStorage,
    focus_manager: FocusManager,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
                trace: Default::default(),
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let renderer = Renderer::new(&device, &queue, config.format, size.width, size.height);

        // Create UI showcasing different Row layouts
        let content = Column::new()
            .push(Label::new("Row Layout Demos").with_color([0.2, 0.6, 1.0, 1.0]))
            .push(Label::new(""))
            // Row with Align::Start
            .push(Label::new("Row with Align::Start:").with_color([0.8, 0.8, 0.8, 1.0]))
            .push(Panel::new(
                Row::new()
                    .push(Button::new("Button 1"))
                    .push(Button::new("Button 2"))
                    .push(Button::new("Button 3"))
                    .align_items(Align::Start),
            ))
            .push(Label::new(""))
            // Row with Align::Center
            .push(Label::new("Row with Align::Center:").with_color([0.8, 0.8, 0.8, 1.0]))
            .push(Panel::new(
                Row::new()
                    .push(Button::new("Button 1"))
                    .push(Button::new("Button 2"))
                    .push(Button::new("Button 3"))
                    .align_items(Align::Center),
            ))
            .push(Label::new(""))
            // Row with Align::End
            .push(Label::new("Row with Align::End:").with_color([0.8, 0.8, 0.8, 1.0]))
            .push(Panel::new(
                Row::new()
                    .push(Button::new("Button 1"))
                    .push(Button::new("Button 2"))
                    .push(Button::new("Button 3"))
                    .align_items(Align::End),
            ))
            .push(Label::new(""))
            // Row with mixed content
            .push(Label::new("Row with equal spacing:").with_color([0.8, 0.8, 0.8, 1.0]))
            .push(Panel::new(
                Row::new()
                    .push(Button::new("Button A"))
                    .push(Button::new("Button B"))
                    .push(Button::new("Button C"))
                    .align_items(Align::Center),
            ))
            .push(Label::new(""))
            // Nested Rows
            .push(Label::new("Nested Rows:").with_color([0.8, 0.8, 0.8, 1.0]))
            .push(Panel::new(
                Column::new()
                    .push(
                        Row::new()
                            .push(Button::new("Top 1"))
                            .push(Button::new("Top 2"))
                            .align_items(Align::Start),
                    )
                    .push(
                        Row::new()
                            .push(Button::new("Bottom 1"))
                            .push(Button::new("Bottom 2"))
                            .push(Button::new("Bottom 3"))
                            .align_items(Align::Center),
                    ),
            ))
            .push(Label::new(""))
            // Mixed content widths
            .push(Label::new("Row with mixed content widths:").with_color([0.8, 0.8, 0.8, 1.0]))
            .push(Panel::new(
                Row::new()
                    .push(Button::new("Short"))
                    .push(Button::new("Medium Button"))
                    .push(Button::new("Very Long Button Text Here"))
                    .align_items(Align::Center),
            ))
            .align_items(Align::Start);

        let root_widget = Panel::new(ScrollView::new(content));

        Self {
            surface,
            device,
            queue,
            config,
            size,
            renderer,
            root_widget: Box::new(root_widget),
            input_state: InputState::default(),
            theme: Theme::dark(),
            window,
            widget_state: WidgetStateStorage::new(),
            focus_manager: FocusManager::new(),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.renderer
                .resize(new_size.width, new_size.height, &self.queue);

            // Layout update
            let rect = Rect::new(0.0, 0.0, new_size.width as f32, new_size.height as f32);
            let request_redraw = || self.window.request_redraw();
            let mut ctx = WidgetContext::new(
                &mut self.renderer,
                &self.theme,
                &mut self.widget_state,
                &mut self.focus_manager,
                &request_redraw,
            );
            ctx.focus_manager.clear_focus_chain();
            self.root_widget.layout(&mut ctx, rect);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        use kirei::gui::core::Event as GuiEvent;

        let gui_event = match event {
            WindowEvent::CursorMoved { position, .. } => {
                let pos = glam::Vec2::new(position.x as f32, position.y as f32);
                self.input_state.mouse_pos = pos;
                Some(GuiEvent::Move(pos))
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.input_state.mouse_pressed = true;
                Some(GuiEvent::Press(self.input_state.mouse_pos))
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.input_state.mouse_pressed = false;
                Some(GuiEvent::Release(self.input_state.mouse_pos))
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_x, y) => *y * 3.0,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.1,
                };
                Some(GuiEvent::Scroll(scroll_delta))
            }
            _ => None,
        };

        if let Some(e) = gui_event {
            let request_redraw = || self.window.request_redraw();
            let mut ctx = WidgetContext::new(
                &mut self.renderer,
                &self.theme,
                &mut self.widget_state,
                &mut self.focus_manager,
                &request_redraw,
            );
            self.root_widget.handle_event(&mut ctx, &e);
            // Always request redraw after handling events (hover, clicks, scroll, etc.)
            self.window.request_redraw();
            true
        } else {
            false
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.theme.colors.background[0] as f64,
                            g: self.theme.colors.background[1] as f64,
                            b: self.theme.colors.background[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        self.renderer.prepare();

        // Layout and draw
        let rect = Rect::new(0.0, 0.0, self.size.width as f32, self.size.height as f32);
        let request_redraw = || self.window.request_redraw();
        let mut ctx = WidgetContext::new(
            &mut self.renderer,
            &self.theme,
            &mut self.widget_state,
            &mut self.focus_manager,
            &request_redraw,
        );
        self.root_widget.layout(&mut ctx, rect);
        self.root_widget.draw(&mut ctx);

        self.renderer.render(
            &self.device,
            &self.queue,
            &view,
            &mut encoder,
            self.size.width,
            self.size.height,
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

struct App {
    state: Option<State>,
}

impl App {
    fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attributes = Window::default_attributes().with_title("Row Layout Demos");
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            let state = pollster::block_on(State::new(window));
            self.state = Some(state);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(state) => state,
            None => return,
        };

        if !state.input(&event) {
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => event_loop.exit(),
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                }
                WindowEvent::RedrawRequested => match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                    Err(e) => eprintln!("{:?}", e),
                },
                _ => {}
            }
        } else {
            // If input handled by GUI, don't call update() to avoid duplicate request_redraw()
            // The input() method already calls request_redraw() if needed
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
