use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::{WebContext, WebViewBuilder}; // Corrected import path and added WebContext

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Minimal Wry-Tao Test")
        .build(&event_loop)
        .expect("Failed to build window");
    
    let mut web_context = WebContext::new(None); // Initialize WebContext
    let _webview = WebViewBuilder::new_with_web_context(&mut web_context)
        .with_url("https://www.google.com")
        .build(&window) // Pass the tao window directly to build()
        .expect("Failed to build webview");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                println!("Wry-Tao application started. WebView initialized.");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Window close requested. Exiting.");
                *control_flow = ControlFlow::Exit
            },
            _ => (),
        }
    });
    Ok(())
}
