use std::time::{Duration, Instant};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window as WinitWindow, WindowBuilder},
};

/// Context passed to your per-frame callback.
/// It gives you the drawing API: `set_background` and `set_rect`.
pub struct FrameCtx<'a> {
    pub width: u32,
    pub height: u32,
    frame: &'a mut [u8],
}

impl<'a> FrameCtx<'a> {
    /// Fill entire background with an RGB color (i64 → u8).
    pub fn set_background(&mut self, r: i64, g: i64, b: i64) {
        let r = r as u8;
        let g = g as u8;
        let b = b as u8;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = ((y * self.width + x) * 4) as usize;
                self.frame[idx] = r;
                self.frame[idx + 1] = g;
                self.frame[idx + 2] = b;
                self.frame[idx + 3] = 255;
            }
        }
    }

    /// Draw a 50x50 white square at (x, y).
    pub fn set_rect(&mut self, x: i64, y: i64) {
        let x = x as i32;
        let y = y as i32;
        let size = 50;

        for j in 0..size {
            for i in 0..size {
                let px = x + i;
                let py = y + j;

                if px >= 0
                    && py >= 0
                    && (px as u32) < self.width
                    && (py as u32) < self.height
                {
                    let idx = (((py as u32) * self.width + (px as u32)) * 4) as usize;
                    self.frame[idx] = 255;
                    self.frame[idx + 1] = 255;
                    self.frame[idx + 2] = 255;
                    self.frame[idx + 3] = 255;
                }
            }
        }
    }
}

/// High-level window wrapper.
/// `F`: per-frame callback, gets a `FrameCtx`.
/// `R`: resize callback, gets (width, height).
pub struct Window<F, R>
where
    F: 'static + for<'a> FnMut(&mut FrameCtx<'a>),
    R: 'static + FnMut(u32, u32),
{
    event_loop: EventLoop<()>,
    window: WinitWindow,
    pixels: Pixels,
    on_frame: F,
    on_resize: R,
    width: u32,
    height: u32,
    last_redraw: Instant,
    frame_time: Duration,
}

impl<F, R> Window<F, R>
where
    F: 'static + for<'a> FnMut(&mut FrameCtx<'a>),
    R: 'static + FnMut(u32, u32),
{
    pub fn new(width: u32, height: u32, on_frame: F, on_resize: R) -> Self {
        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title("Interpreter window")
            .with_inner_size(LogicalSize::new(width as f64, height as f64))
            .build(&event_loop)
            .unwrap();

        let size = window.inner_size();
        let width = size.width;
        let height = size.height;

        let surface = SurfaceTexture::new(width, height, &window);
        let pixels = Pixels::new(width, height, surface).expect("failed to create pixels");

        Self {
            event_loop,
            window,
            pixels,
            on_frame,
            on_resize,
            width,
            height,
            last_redraw: Instant::now(),
            frame_time: Duration::from_millis(16), // ~60 FPS
        }
    }

    pub fn run(self) -> ! {
        let Window {
            event_loop,
            window,
            mut pixels,
            mut on_frame,
            mut on_resize,
            mut width,
            mut height,
            mut last_redraw,
            frame_time,
        } = self;

        event_loop.run(move |event, _target, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(new_size) => {
                        // Update logical framebuffer size
                        width = new_size.width.max(1);
                        height = new_size.height.max(1);

                        // 1) Resize the *buffer* (changes frame.len())
                        if let Err(e) = pixels.resize_buffer(width, height) {
                            eprintln!("resize_buffer error: {e}");
                            *control_flow = ControlFlow::Exit;
                            return;
                        }

                        // 2) Resize the *surface* (OS window / swapchain)
                        if let Err(e) = pixels.resize_surface(width, height) {
                            eprintln!("resize_surface error: {e}");
                            *control_flow = ControlFlow::Exit;
                            return;
                        }

                        // Notify user code (interpreter) of new size
                        on_resize(width, height);
                    }
                    _ => {}
                },

                Event::MainEventsCleared => {
                    // ~60 FPS
                    if last_redraw.elapsed() >= frame_time {
                        window.request_redraw();
                        last_redraw = Instant::now();
                    }
                }

                Event::RedrawRequested(_) => {
                    // Avoid drawing when minimized (0x0)
                    if width == 0 || height == 0 {
                        return;
                    }

                    let frame = pixels.frame_mut();
                    let mut ctx = FrameCtx {
                        width,
                        height,
                        frame,
                    };

                    on_frame(&mut ctx);

                    if let Err(e) = pixels.render() {
                        eprintln!("pixels error: {e}");
                        *control_flow = ControlFlow::Exit;
                    }
                }

                _ => {}
            }
        });
    }
}
