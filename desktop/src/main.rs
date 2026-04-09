use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId}
};
use pixels::{Pixels, SurfaceTexture};
use chip8_core::*;
use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;


const TICK_PER_FRAME: usize = 10;

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    emu: Option<Emu>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(event_loop.create_window(Window::default_attributes()
                .with_title("Chip8 Emulator")).unwrap());

            // Create a pixel buffer
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, Arc::clone(&window));
            let pixels = Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture).unwrap();

            self.window = Some(window);
            self.pixels = Some(pixels);
        }
    }

    fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _window_id: WindowId,
            event: WindowEvent,
        ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let emu = self.emu.as_mut().unwrap();

                for _ in 0..TICK_PER_FRAME {
                    emu.tick();
                }

                emu.timer_tick();
                
                let pixels = self.pixels.as_mut().unwrap();
                draw(emu, pixels.frame_mut());

                if let Err(err) = pixels.render() {
                    println!("Render error: {}", err);
                    event_loop.exit();
                }

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

fn draw(emu: &Emu, frame: &mut [u8]) {
    let screen_buf = emu.get_display();
    for (c, pix) in screen_buf.iter().zip(frame.chunks_exact_mut(4)) {
        let mut rgba = [0, 0, 0, 0xFF];
        if *c {
            rgba = [0xFF, 0xFF, 0xFF, 0xFF];
        }   
        pix.copy_from_slice(&rgba);
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    let mut chip8 = Emu::new();

    let mut rom = File::open(&args[1]).expect("Unable to read file.");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App{
        window: None,
        pixels: None,
        emu: Some(chip8)
    };
    _ = event_loop.run_app(&mut app);
}
