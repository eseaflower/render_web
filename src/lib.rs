/**


Changes in .vsconfig to use the wasm32-target for rust-analyzer!!!!!!!!!!!!


**/
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;
use winit::platform::web::WindowBuilderExtWebSys;
use winit::platform::web::WindowExtWebSys;

use image::{self, EncodableLayout};
use log::{self, info};
use simple_logger;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod render_target;
mod vertex;
mod view_state;
use render_target::{SwapchainTarget, TextureTarget};
mod renderer;
use raw_window_handle::HasRawWindowHandle;
use renderer::State;
use std::sync::mpsc::channel;

async fn create_for_window(window: &Window) -> State<SwapchainTarget> {
    //let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let instance = wgpu::Instance::new();
    let surface = unsafe { instance.create_surface(window) };
    let size = window.inner_size();

    let format = wgpu::TextureFormat::Bgra8Unorm; // WASM

    let target = SwapchainTarget::new(surface, format);

    State::new(instance, (size.width, size.height), target).await
}

async fn create_for_handle(window: &CanvasWindow, size: (u32, u32)) -> State<SwapchainTarget> {
    //let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let instance = wgpu::Instance::new();
    let surface = unsafe { instance.create_surface(window) };

    let format = wgpu::TextureFormat::Bgra8Unorm; // WASM

    let target = SwapchainTarget::new(surface, format);

    State::new(instance, (size.0, size.1), target).await
}

fn create_window() -> (
    winit::event_loop::EventLoop<StateSetup>,
    winit::event_loop::EventLoopProxy<StateSetup>,
    winit::window::Window,
) {
    let event_loop = EventLoop::with_user_event();
    let proxy = event_loop.create_proxy();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // WASM

    // Actually create the canvas element (?)
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.body())
        .and_then(|body| {
            body.append_child(&web_sys::Element::from(window.canvas()))
                .ok()
        })
        .expect("couldn't append canvas to document body");

    info!("Window created");
    (event_loop, proxy, window)
}

fn run_event_loop(event_loop: winit::event_loop::EventLoop<StateSetup>) {
    let mut mouse_down = false;
    let mut ctrl_down = false;

    let (mut mx, mut my) = (100.0_f32, 100.0_f32);
    let mut delta = 3.5_f32;

    //let mut last_now = Instant::now();
    let perf = Some(web_sys::window().unwrap().performance().unwrap());

    let mut last_frame = 0.0;

    let mut setup = None;

    event_loop.run(move |event, _, control_flow| {
        // Wait for the state setup to be complete
        let event = match setup {
            None => match event {
                Event::UserEvent(mut s) => {
                    s.state.swap_image();
                    setup = Some(s);
                    info!("Got StateSetup");
                    return;
                }
                _ => return, // Only interested in setup msgs until complete.
            },
            _ => event,
        };

        *control_flow = ControlFlow::Poll;

        // Get the state. It is safe to unwrap since we have matched on the setup above.
        let setup = setup.as_mut().unwrap();
        let state = &mut setup.state;
        let window = &mut setup.window;

        // Normal event-loop
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            ..
                        } => {
                            state.swap_image();
                        }
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        setup
                            .state
                            .resize((physical_size.width, physical_size.height));
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        setup
                            .state
                            .resize((new_inner_size.width, new_inner_size.height));
                    }
                    WindowEvent::CursorMoved {
                        position,
                        modifiers,
                        ..
                    } => {
                        // For web...
                        match modifiers {
                            &ModifiersState::CTRL => ctrl_down = true,
                            _ => ctrl_down = false,
                        };

                        if mouse_down {
                            if ctrl_down {
                                state.update_zoom((position.x as f32, position.y as f32));
                            } else {
                                state.update_position((position.x as f32, position.y as f32));
                                //info!("{:?}", position);
                            }
                            //state.render();
                            //window.request_redraw();
                            let n = if let Some(p) = &perf { p.now() } else { 0.0 };
                            let diff = n - last_frame;
                            last_frame = n;
                            //info!("Frame time: {}", diff);
                        }
                    }
                    WindowEvent::MouseInput {
                        state: elem_state,
                        button,
                        ..
                    } => match button {
                        MouseButton::Left => match elem_state {
                            ElementState::Pressed => mouse_down = true,
                            _ => {
                                mouse_down = false;
                                state.clear_anchor();
                            }
                        },
                        _ => {}
                    },
                    WindowEvent::ModifiersChanged(modifier) => match modifier {
                        &ModifiersState::CTRL => ctrl_down = true,
                        _ => ctrl_down = false,
                    },
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                // let current_now = Instant::now();
                // let frame_time: Duration = current_now - last_now;
                // if frame_time.as_millis() > 0 {
                if state.is_dirty() {
                    state.render();
                }
                //     last_now = current_now;
                //     println!("Frame time: {}", frame_time.as_millis());
                // }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                // if state.is_dirty() {
                //     window.request_redraw();
                // }
                if mx > 300_f32 {
                    delta *= -1.0_f32;
                } else if mx < -100.0_f32 {
                    delta *= -1.0_f32;
                }
                mx += delta;
                state.update_position((mx, my));
                window.request_redraw();
                //state.render();
            }
            _ => {}
        }
    });
}

struct StateSetup {
    window: winit::window::Window,
    state: State<SwapchainTarget>,
}

async fn run_setup(
    window: winit::window::Window,
    proxy: winit::event_loop::EventLoopProxy<StateSetup>,
) {
    // Run the async methods here
    let state = create_for_window(&window).await;
    // Post the result to the event-loop.
    proxy.send_event(StateSetup { window, state });
}

#[wasm_bindgen]
pub fn entry() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("could not initialize logger");

    info!("First log!");
    let (event_loop, proxy, window) = create_window();
    wasm_bindgen_futures::spawn_local(run_setup(window, proxy));

    //let (event_loop, window, state) = block_on(create_window());
    run_event_loop(event_loop);

    // TODO: Don't run the event loop in the future (see https://github.com/gfx-rs/wgpu-rs/pull/508)
    // Instead split the setup-part (that needs to be async) from the running of the event loop.
    //wasm_bindgen_futures::spawn_local(run_window());
}

#[wasm_bindgen]
struct RenderController {
    state: State<SwapchainTarget>,
}

struct CanvasWindow {
    id: u32,
}

unsafe impl HasRawWindowHandle for CanvasWindow {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        let handle = raw_window_handle::web::WebHandle {
            id: self.id,
            ..raw_window_handle::web::WebHandle::empty()
        };
        raw_window_handle::RawWindowHandle::Web(handle)
    }
}

#[wasm_bindgen]
impl RenderController {
    pub async fn new(canvas_id: u32, width: u32, height: u32) -> RenderController {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");

        let window = CanvasWindow { id: canvas_id };
        let mut state = create_for_handle(&window, (width, height)).await;

        RenderController { state }

        // let canvas = web_sys::window()
        //     .unwrap()
        //     .document()
        //     .unwrap()
        //     .get_element_by_id(&canvas_id)
        //     .unwrap()
        //     .dyn_into::<web_sys::HtmlCanvasElement>();
        // log::info!("Canvas is: {:?}", canvas);

        // let event_loop = winit::event_loop::EventLoop::<StateSetup>::with_user_event();
        // let proxy = event_loop.create_proxy();
        // let window = winit::window::WindowBuilder::new()
        //     .with_canvas(canvas.ok())
        //     .build(&event_loop)
        //     .unwrap();

        // //run_setup(window, proxy).await;
        // run_event_loop(event_loop);
    }

    pub fn render(&mut self) {
        self.state.render();
        log::info!("Render called");
    }

    pub fn swap_image(&mut self) {
        self.state.swap_image();
    }

    pub fn update_position(&mut self, x: f32, y: f32) {
        self.state.update_position((x, y));
    }

    pub fn update_zoom(&mut self, x: f32, y: f32) {
        self.state.update_zoom((x, y));
    }

    pub fn clear_anchor(&mut self) {
        self.state.clear_anchor();
    }

    pub fn set_viewport_size(&mut self, width: u32, height: u32) {
        self.state.resize((width, height));
    }
}
