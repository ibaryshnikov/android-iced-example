use std::sync::Arc;

use iced_wgpu::graphics::Viewport;
use iced_wgpu::{wgpu, Engine, Renderer};
use iced_winit::core::{mouse, renderer, window, Color, Font, Pixels, Size, Theme};
use iced_winit::runtime::{program, Debug};
use iced_winit::{conversion, winit, Clipboard};
use wgpu::{Device, Instance, Queue, TextureFormat};
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::ModifiersState;
use winit::platform::android::activity::AndroidApp;
use winit::platform::android::EventLoopBuilderExtAndroid;
use winit::window::{Window, WindowId};

mod controls;
mod scene;

use controls::Controls;
use scene::Scene;

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Info));

    log::info!("android_main started");

    let event_loop = EventLoop::builder()
        .with_android_app(app)
        .build()
        .expect("Should build event loop");

    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Should run event loop");
}

struct App {
    app_data: Option<AppData>,
    resized: bool,
    request_redraw: bool,
    wait_cancelled: bool,
    close_requested: bool,
    cursor_position: Option<winit::dpi::PhysicalPosition<f64>>,
    modifiers: ModifiersState,
}

struct AppData {
    state: program::State<Controls>,
    scene: Scene,
    window: Arc<Window>,
    device: Device,
    queue: Queue,
    surface: wgpu::Surface<'static>,
    format: TextureFormat,
    engine: Engine,
    renderer: Renderer,
    clipboard: Clipboard,
    viewport: Viewport,
    debug: Debug,
}

impl App {
    fn new() -> Self {
        Self {
            app_data: None,
            resized: false,
            request_redraw: false,
            wait_cancelled: false,
            close_requested: false,
            cursor_position: None,
            modifiers: ModifiersState::default(),
        }
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {}

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.app_data.is_some() {
            log::info!("Already initialized, skipping");
            return;
        }

        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let attrs = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        let physical_size = window.inner_size();
        let viewport = Viewport::with_physical_size(
            Size::new(physical_size.width, physical_size.height),
            window.scale_factor(),
        );
        let clipboard = Clipboard::connect(&window);

        let surface = instance
            .create_surface(window.clone())
            .expect("Create window surface");

        let (format, adapter, device, queue) = pollster::block_on(async {
            let adapter =
                wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
                    .await
                    .expect("Create adapter");

            let adapter_features = adapter.features();

            let capabilities = surface.get_capabilities(&adapter);

            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: adapter_features & wgpu::Features::default(),
                        required_limits: wgpu::Limits::default(),
                    },
                    None,
                )
                .await
                .expect("Request device");

            (
                capabilities
                    .formats
                    .iter()
                    .copied()
                    .find(wgpu::TextureFormat::is_srgb)
                    .or_else(|| capabilities.formats.first().copied())
                    .expect("Get preferred format"),
                adapter,
                device,
                queue,
            )
        });

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: physical_size.width,
                height: physical_size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            },
        );

        let scene = Scene::new(&device, format);
        let controls = Controls::new();

        let mut debug = Debug::new();
        let engine = Engine::new(&adapter, &device, &queue, format, None);
        let mut renderer = Renderer::new(&device, &engine, Font::default(), Pixels::from(16));

        let state =
            program::State::new(controls, viewport.logical_size(), &mut renderer, &mut debug);

        event_loop.set_control_flow(ControlFlow::Wait);

        self.cursor_position = None;
        self.modifiers = ModifiersState::default();

        let app_data = AppData {
            state,
            scene,
            window,
            device,
            queue,
            surface,
            format,
            engine,
            renderer,
            clipboard,
            viewport,
            debug,
        };
        self.app_data = Some(app_data);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        log::info!("Window event: {:?}", event);

        let Some(app_data) = self.app_data.as_mut() else {
            return;
        };

        let AppData {
            state,
            scene,
            window,
            device,
            queue,
            surface,
            format,
            engine,
            renderer,
            clipboard,
            debug,
            ..
        } = app_data;

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if self.resized {
                    let size = window.inner_size();

                    app_data.viewport = Viewport::with_physical_size(
                        Size::new(size.width, size.height),
                        window.scale_factor(),
                    );

                    surface.configure(
                        device,
                        &wgpu::SurfaceConfiguration {
                            format: *format,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::AutoVsync,
                            alpha_mode: wgpu::CompositeAlphaMode::Auto,
                            view_formats: vec![],
                            desired_maximum_frame_latency: 2,
                        },
                    );

                    self.resized = false;
                }

                match surface.get_current_texture() {
                    Ok(frame) => {
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        let program = state.program();

                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        {
                            let mut render_pass =
                                Scene::clear(&view, &mut encoder, program.background_color());
                            scene.draw(&mut render_pass);
                        }

                        renderer.present::<String>(
                            engine,
                            device,
                            queue,
                            &mut encoder,
                            None,
                            frame.texture.format(),
                            &view,
                            &app_data.viewport,
                            &[],
                        );

                        engine.submit(queue, encoder);
                        frame.present();

                        window.set_cursor(iced_winit::conversion::mouse_interaction(
                            state.mouse_interaction(),
                        ));
                    }
                    Err(error) => match error {
                        wgpu::SurfaceError::OutOfMemory => {
                            panic!(
                                "Swapchain error: {error}. \
                            Rendering cannot continue."
                            )
                        }
                        _ => {
                            window.request_redraw();
                        }
                    },
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = Some(position);
            }
            WindowEvent::Touch(touch) => {
                self.cursor_position = Some(touch.location);
            }
            WindowEvent::Resized(_) => {
                self.resized = true;
            }
            _ => (),
        }

        if let Some(event) = iced_winit::conversion::window_event(
            window::Id::MAIN,
            event,
            window.scale_factor(),
            self.modifiers,
        ) {
            state.queue_event(event);
        }

        if !state.is_queue_empty() {
            let _ = state.update(
                app_data.viewport.logical_size(),
                self.cursor_position
                    .map(|p| conversion::cursor_position(p, app_data.viewport.scale_factor()))
                    .map(mouse::Cursor::Available)
                    .unwrap_or(mouse::Cursor::Unavailable),
                renderer,
                &Theme::Dark,
                &renderer::Style {
                    text_color: Color::WHITE,
                },
                clipboard,
                debug,
            );

            window.request_redraw();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.request_redraw && !self.wait_cancelled && !self.close_requested {
            if let Some(app_data) = &self.app_data {
                app_data.window.request_redraw();
            }
            self.request_redraw = false;
        }

        if self.close_requested {
            event_loop.exit();
        }
    }
}