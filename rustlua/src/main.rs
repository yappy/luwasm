mod emscripten;
mod sdl;

use std::cell::RefCell;

use log::{info, trace};
use mlua::{Lua, LuaOptions, StdLib};

fn print_test() {
    println!("println to stdout");
    eprintln!("eprintln to stderr");

    emscripten::log(emscripten::LogTarget::ModuleOut, "Module stdout");
    emscripten::log(emscripten::LogTarget::ModuleErr, "Module stderr");
    emscripten::log(emscripten::LogTarget::ConsoleError, "console.error");
    emscripten::log(emscripten::LogTarget::ConsoleWarn, "console.warn");
    emscripten::log(emscripten::LogTarget::ConsoleInfo, "console.info");
    emscripten::log(emscripten::LogTarget::ConsoleDebug, "console.debug");

    log::error!("log::error");
    log::warn!("log::warn");
    log::info!("log::info");
    log::debug!("log::debug");
    log::trace!("log::trace");
}

fn lua_test() -> anyhow::Result<()> {
    lua_exec("print('Hello Lua!')")?;
    lua_exec("this is not lua source")?;

    Ok(())
}

fn lua_exec(src: &str) -> anyhow::Result<()> {
    let libs = StdLib::ALL_SAFE;
    let options = LuaOptions::new().catch_rust_panics(true);
    let lua = Lua::new_with(libs, options)?;

    /*
     * source: the source of the chunk that created the function.
     * If source starts with a '@', it means that the function was defined
     * in a file where the file name follows the '@'.
     * If source starts with a '=', the remainder of its contents describes
     * the source in a user-dependent manner.
     * Otherwise, the function was defined in a string
     * where source is that string.
     */
    // not compiled yet
    // by default, Rust file:line:column (for panic) will be used for name
    let chunk = lua.load(src).set_name("=<STR_SRC>");
    // compile and execute
    chunk.exec()?;

    Ok(())
}

fn set_callback_button_clicked() {
    let res = emscripten::set_click_callback("#run", |_, _| {
        println!("clicked");
        let src = emscripten::eval_js(
            r"
(function() {
    if (!document) return null;
    var e = document.getElementById('source');
    if (!e) return null;
    return e.value;
})()
",
        );
        let src = if let Some(src) = src {
            src
        } else {
            eprintln!("Get source failed");
            return false;
        };

        match lua_exec(&src) {
            Ok(()) => {
                println!("Lua executed successfully");
                println!();
            }
            Err(err) => {
                println!("{err}");
                println!();
            }
        }

        false
    });
    if let Err(err) = res {
        println!("{err}");
    } else {
        println!("set_click_callback ok");
    }
}

fn ls(dir: impl AsRef<::std::path::Path>) -> anyhow::Result<()> {
    for entry in ::std::fs::read_dir(&dir)? {
        let entry = entry?;
        let ftype = entry.file_type()?;
        if ftype.is_dir() {
            println!("D {}", entry.path().to_string_lossy());
            ls(entry.path())?;
        } else if ftype.is_file() {
            println!("F {}", entry.path().to_string_lossy());
        }
    }
    Ok(())
}

fn fs_test() -> anyhow::Result<()> {
    println!("[FS Test]");
    let pwd = ::std::env::current_dir()?;
    println!("pwd: {}", pwd.to_string_lossy());
    let home = ::std::env::var("HOME")?;
    println!("$HOME: {home}");
    ls("/")?;

    Ok(())
}

fn update() {}

fn draw(surface: &sdl::Surface) {
    static COUNT: ::std::sync::atomic::AtomicI32 = ::std::sync::atomic::AtomicI32::new(0);
    let count = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let val = (count & 0xff) as u8;
    for p in surface.pixels() {
        *p = val;
    }
}

fn main_loop(surface: &sdl::Surface) {
    update();

    if surface.must_lock() {
        surface.lock().expect("lock failed");
    }
    draw(surface);
    if surface.must_lock() {
        surface.unlock();
    }
    surface.flip().expect("flip failed");
}

fn main_loop_raw(surface: &sdl::Surface) {
    struct FpsState {
        fps: f64,
        start_time: f64,
        frame_count: u32,
    }
    thread_local! {
        static FPS_STATE: RefCell<FpsState> = RefCell::new(FpsState {
            fps: 0.0,
            start_time: 0.0,
            frame_count: 0,
        });
    }

    const FPS_UPDATE_PERIOD_MS: f64 = 2000.0;

    FPS_STATE.with(|cell| {
        let mut state = cell.borrow_mut();

        let now = emscripten::performance_now();
        if state.start_time == 0.0 {
            // first call
            state.start_time = now;
            state.frame_count = 0;
        } else {
            state.frame_count += 1;
        }
        let elapsed = now - state.start_time;
        if elapsed >= FPS_UPDATE_PERIOD_MS {
            state.fps = (state.frame_count as f64) / elapsed * 1000.0;
            state.start_time = now;
            state.frame_count = 0;
            trace!("fps: {:.1}", state.fps);
        }
    });

    main_loop(surface);
}

fn setup_main_loop() -> anyhow::Result<()> {
    sdl::init(sdl::init::SDL_INIT_VIDEO)?;
    let surface = sdl::set_video_mode(
        640,
        480,
        32,
        sdl::flags::SDL_SWSURFACE | sdl::flags::SDL_DOUBLEBUF,
    )?;

    // fps (not 0) does not work well
    // probably because of security issue?
    // fps=0 means to use requestAnimationFrame()
    emscripten::set_main_loop(0, move || {
        main_loop_raw(&surface);
    });

    Ok(())
}

fn run() -> anyhow::Result<()> {
    print_test();
    set_callback_button_clicked();
    if let Err(err) = lua_test() {
        eprintln!("{err}");
    }
    fs_test()?;

    setup_main_loop()?;

    Ok(())
}

fn main() {
    emscripten::setup_logger(log::LevelFilter::Trace);

    match run() {
        Ok(()) => {}
        Err(err) => {
            log::error!("{err:#}");
            eprintln!("{err:#}");
        }
    }
    /*
     * By default Emscripten sets EXIT_RUNTIME=0,
     * which means that we don’t include code to shut down the runtime.
     * That means that when main() exits, we don’t flush the stdio streams,
     * or call the destructors of global C++ objects, or call atexit callbacks.
     * This lets us emit smaller code by default,
     * and is normally what you want on the web:
     * even though main() exited, you may have something
     * asynchronous happening later that you want to execute.
     */
    info!("main exit (keep alive)");
}
