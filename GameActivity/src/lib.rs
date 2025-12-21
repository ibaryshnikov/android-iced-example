use std::sync::Arc;
use std::time::Instant;

use iced_wgpu::graphics::{Shell, Viewport};
use iced_wgpu::{wgpu, Engine, Renderer};
use iced_winit::core::{mouse, renderer, window, Event, Font, Pixels, Size, Theme};
use iced_winit::runtime::user_interface::{self, UserInterface};
use iced_winit::{conversion, winit};
use log::LevelFilter;
use wgpu::{Device, Queue, TextureFormat};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::platform::android::activity::AndroidApp;
use winit::platform::android::EventLoopBuilderExtAndroid;
use winit::window::{Window, WindowId};

mod clipboard;
mod controls;
mod java;
mod scene;

use clipboard::Clipboard;
use controls::Controls;
use scene::Scene;

// winit ime support
// https://github.com/rust-windowing/winit/pull/2993

// issue with android-activity crate default_motion_filter function
// https://github.com/rust-mobile/android-activity/issues/79

#[no_mangle]
fn android_main(android_app: AndroidApp) {
    let logger_config = android_logger::Config::default().with_max_level(LevelFilter::Info);
    android_logger::init_once(logger_config);

    log::info!("android_main started");

    let event_loop = EventLoop::with_user_event()
        .with_android_app(android_app)
        .build()
        .expect("Should build event loop");

    let proxy = event_loop.create_proxy();
    let controls = Controls::new(proxy);

    let mut app = App::new(controls);
    event_loop.run_app(&mut app).expect("Should run event loop");
}

#[derive(Debug)]
enum UserEvent {
    ShowKeyboard,
    HideKeyboard,
}

struct App {
    app_data: Option<AppData>,
    resized: bool,
    cursor: mouse::Cursor,
    modifiers: ModifiersState,
    events: Vec<Event>,
    cache: user_interface::Cache,
    controls: Controls,
}

struct AppData {
    scene: Scene,
    window: Arc<Window>,
    device: Device,
    queue: Queue,
    surface: wgpu::Surface<'static>,
    format: TextureFormat,
    renderer: Renderer,
    clipboard: Clipboard,
    viewport: Viewport,
}

impl App {
    fn new(controls: Controls) -> Self {
        Self {
            app_data: None,
            resized: false,
            cursor: mouse::Cursor::Unavailable,
            modifiers: ModifiersState::default(),
            events: Vec::new(),
            cache: user_interface::Cache::new(),
            controls,
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
        // log::info!("New events cause {:?}", cause);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("Resumed");
        // if self.app_data.is_some() {
        //     log::info!("Already initialized, skipping");
        //     return;
        // }

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());

        let attrs = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        window.set_ime_allowed(true);

        let physical_size = window.inner_size();
        let viewport = Viewport::with_physical_size(
            Size::new(physical_size.width, physical_size.height),
            window.scale_factor() as f32,
        );
        let clipboard = Clipboard {};

        let surface = instance
            .create_surface(window.clone())
            .expect("Create window surface");

        let (format, adapter, device, queue) = futures::executor::block_on(async {
            let adapter =
                wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
                    .await
                    .expect("Create adapter");

            let adapter_features = adapter.features();

            let capabilities = surface.get_capabilities(&adapter);

            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: None,
                    required_features: adapter_features & wgpu::Features::default(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                    trace: wgpu::Trace::Off,
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                })
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

        let engine = Engine::new(
            &adapter,
            device.clone(),
            queue.clone(),
            format,
            None,
            Shell::headless(),
        );
        let renderer = Renderer::new(engine, Font::default(), Pixels::from(16));

        event_loop.set_control_flow(ControlFlow::Wait);

        self.modifiers = ModifiersState::default();

        let app_data = AppData {
            scene,
            window,
            device,
            queue,
            surface,
            format,
            renderer,
            clipboard,
            viewport,
        };
        self.app_data = Some(app_data);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::ShowKeyboard => {
                java::call_instance_method("showKeyboard");
            }
            UserEvent::HideKeyboard => {
                java::call_instance_method("hideKeyboard");
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        log::info!("DeviceEvent {:?}", event);
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
            scene,
            window,
            device,
            queue,
            surface,
            format,
            renderer,
            clipboard,
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
                        window.scale_factor() as f32,
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

                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        {
                            let mut render_pass =
                                Scene::clear(&view, &mut encoder, self.controls.background_color());
                            scene.draw(&mut render_pass);
                        }

                        queue.submit([encoder.finish()]);

                        let mut interface = UserInterface::build(
                            self.controls.view(),
                            app_data.viewport.logical_size(),
                            std::mem::take(&mut self.cache),
                            renderer,
                        );

                        let (state, _) = interface.update(
                            &[Event::Window(
                                window::Event::RedrawRequested(Instant::now()),
                            )],
                            self.cursor,
                            renderer,
                            clipboard,
                            &mut Vec::new(),
                        );

                        if let user_interface::State::Updated {
                            mouse_interaction, ..
                        } = state
                        {
                            if let Some(icon) = conversion::mouse_interaction(mouse_interaction) {
                                window.set_cursor(icon);
                                window.set_cursor_visible(true);
                            } else {
                                window.set_cursor_visible(false);
                            }
                        }

                        let theme = Theme::Ferra;
                        interface.draw(
                            renderer,
                            &theme,
                            &renderer::Style {
                                text_color: theme.palette().text,
                            },
                            self.cursor,
                        );
                        self.cache = interface.into_cache();

                        renderer.present(None, frame.texture.format(), &view, &app_data.viewport);

                        frame.present();
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
                self.cursor = mouse::Cursor::Available(conversion::cursor_position(
                    position,
                    app_data.viewport.scale_factor(),
                ));
            }
            WindowEvent::Touch(touch) => {
                self.cursor = mouse::Cursor::Available(conversion::cursor_position(
                    touch.location,
                    app_data.viewport.scale_factor(),
                ));
            }
            WindowEvent::Ime(ref ime) => {
                log::info!("Ime event: {:?}", ime);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                ref event,
                is_synthetic: _,
            } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    match code {
                        KeyCode::ShiftLeft | KeyCode::ShiftRight => match event.state {
                            ElementState::Pressed => self.modifiers |= ModifiersState::SHIFT,
                            ElementState::Released => self.modifiers &= !ModifiersState::SHIFT,
                        },
                        KeyCode::ControlLeft | KeyCode::ControlRight => match event.state {
                            ElementState::Pressed => self.modifiers |= ModifiersState::CONTROL,
                            ElementState::Released => self.modifiers &= !ModifiersState::CONTROL,
                        },
                        _ => (),
                    }
                }
            }
            WindowEvent::Resized(_) => {
                self.resized = true;
            }
            _ => (),
        }

        if let Some(event) =
            conversion::window_event(event, window.scale_factor() as f32, self.modifiers)
        {
            self.events.push(event);
        }

        if self.events.is_empty() {
            return;
        }
        let mut interface = UserInterface::build(
            self.controls.view(),
            app_data.viewport.logical_size(),
            std::mem::take(&mut self.cache),
            renderer,
        );

        let mut messages = Vec::new();

        let _ = interface.update(
            &self.events,
            self.cursor,
            renderer,
            clipboard,
            &mut messages,
        );

        self.events.clear();
        self.cache = interface.into_cache();

        for message in messages {
            let _ = self.controls.update(message);
        }

        window.request_redraw();
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}
}
