mod lib_interpreter;
mod window; //interpreter module
#[allow(unused)]
use std::sync::Arc;
#[allow(unused)]
use lib_interpreter::{Interpreter, Value};
use window::Window;

fn main() -> Result<(), String> {
    // 1) Create interpreter from file
    let interpreter = Arc::new(Interpreter::from_file("script.txt")?);

    // 2) Run interpreter in its own thread
    Interpreter::run_in_thread(interpreter.clone());

    // 3) Clone (reference to the interpeter) for callbacks
    let interpreter_ref_1 = interpreter.clone();

    // 4) Create window
    let win = Window::new(
        500,
        500,
        // ---- runs on each frame draw ----
        move |ctx| {
            
            let r = interpreter_ref_1.get_var("r").map(|v| v.unwrap_int()).unwrap_or(0);
            let g = interpreter_ref_1.get_var("g").map(|v| v.unwrap_int()).unwrap_or(0);
            let b = interpreter_ref_1.get_var("b").map(|v| v.unwrap_int()).unwrap_or(0);
            ctx.set_background(r, g, b);

            let x = interpreter_ref_1.get_var("x").map(|v| v.unwrap_int()).unwrap_or(50);
            let y = interpreter_ref_1.get_var("y").map(|v| v.unwrap_int()).unwrap_or(50);
            ctx.set_rect(x, y);
            
        },
        // ---- runs on window resize ----
        move |w, h| {
            interpreter.set_var("w", Value::Int(w as i64));
            interpreter.set_var("h", Value::Int(h as i64));
        },
    );

    // 5) Start the event loop (never returns)
    win.run();
}
