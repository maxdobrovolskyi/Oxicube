//winit
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use winit::dpi::LogicalSize;
//tacing
use tracing::info;

struct App {
    window: Option<Box<dyn Window>>,
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();
        let _ = window.request_surface_size(LogicalSize::new(1280.0, 720.0).into());
        let _ = window.set_title("Oxicube");
        self.window = Some(window);
    }
    fn window_event(
            &mut self,
            event_loop: &dyn ActiveEventLoop,
            window_id: WindowId,
            event: WindowEvent,
        ) {
        match event {
            WindowEvent::RedrawRequested => {
                println!("{:#?}", window_id)
            }
            WindowEvent::CloseRequested => {
                println!("Close window");
                event_loop.exit();
            }
            WindowEvent::SurfaceResized(size) => {
                print!("resized: {:#?}", size);
            }
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                if let Some(text) = &event.text {
                    println!("key={:?} state={:?} text={:?}",
                        event.physical_key,
                        event.state,
                        text
                    );
                    if text == "\u{1b}" {
                        println!("Close window");
                        event_loop.exit();
                    }
                }
            }

            _ => {}
        }
    }
}

fn main() {
    //init logging
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .pretty()
        .init();

    info!("App started");

    let event_loop = EventLoop::new().unwrap();
    let app = App { window: None };
    
    event_loop.run_app(Box::leak(Box::new(app))).unwrap();
}