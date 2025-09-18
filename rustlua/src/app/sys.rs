use crate::emapi;
use std::cell::RefCell;

const HOME_DIR: &str = "/home/web_user";

pub fn run() -> anyhow::Result<()> {
    println!("cd {HOME_DIR}");
    if let Err(err) = std::env::set_current_dir(HOME_DIR) {
        println!("Change working directory failed");
        log::error!("{err}");
    }

    set_callback_button_clicked();

    setup_main_loop()?;

    Ok(())
}

fn process_cmdline() -> anyhow::Result<()> {
    let cmdline = emapi::emscripten::eval_js_string(
        r"
(() => {
    try { return Module.takeCommandLine(); }
    catch (e) { console.error(e); return null; }
})()",
    );
    if cmdline.is_none() {
        return Ok(());
    }
    let cmdline = cmdline.unwrap();
    log::info!("EXEC: {cmdline}");

    super::cmdline::exec(&cmdline)
}

fn process_import_file() -> anyhow::Result<()> {
    let file_name = emapi::emscripten::eval_js_string(
        r"
(() => {
    try { return Module.takeImportFileName(); }
    catch (e) { console.error(e); return null; }
})()",
    );
    let file_data = emapi::emscripten::eval_js_string(
        r"
(() => {
    try { return Module.takeImportFileData(); }
    catch (e) { console.error(e); return null; }
})()",
    );

    if file_name.is_none() || file_data.is_none() {
        return Ok(());
    }
    let file_name = file_name.unwrap();
    let file_data = file_data.unwrap();
    anyhow::ensure!(
        file_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".-_~".find(c).is_some()),
        "Invalid file name: {file_name}"
    );
    log::info!("Import: {file_name}");

    // get start index of BASE64STRING in "data:*/*;base64,BASE64STRING..."
    anyhow::ensure!(file_data.starts_with("data:"), "Invalid data URL");
    let pat = ";base64,";
    let base64_start = if let Some(ind) = file_data.find(pat) {
        ind + pat.len()
    } else {
        anyhow::bail!("Invalid data URL");
    };
    let base64_string = &file_data[base64_start..];
    log::debug!("{base64_string}");

    // base64 decode
    let bin = {
        use base64::Engine;
        use base64::prelude::*;
        BASE64_STANDARD.decode(base64_string)?
    };

    let path: &::std::path::Path = HOME_DIR.as_ref();
    ::std::fs::write(path.join(&file_name), &bin)?;

    log::info!("Import: {file_name} (size={})", bin.len());

    Ok(())
}

pub fn lua_exec(src: &str) -> anyhow::Result<()> {
    let libs = mlua::StdLib::ALL_SAFE;
    let options = mlua::LuaOptions::new().catch_rust_panics(true);
    let lua = mlua::Lua::new_with(libs, options)?;

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
    let res = emapi::emscripten::set_click_callback("#run", |_, _| {
        println!("clicked");
        let src = emapi::emscripten::eval_js_string(
            r"
(() => {
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

fn update() {
    if let Err(err) = process_cmdline() {
        println!("{err:#}");
    }
    if let Err(err) = process_import_file() {
        eprintln!("{err:#}");
    }
}

fn render(surface: &emapi::sdl::Surface) {
    static COUNT: ::std::sync::atomic::AtomicI32 = ::std::sync::atomic::AtomicI32::new(0);
    let count = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let val = (count & 0xff) as u8;
    for p in surface.pixels() {
        *p = val;
    }
}

fn main_loop(surface: &emapi::sdl::Surface) {
    update();

    if surface.must_lock() {
        surface.lock().expect("lock failed");
    }
    render(surface);
    if surface.must_lock() {
        surface.unlock();
    }
}

fn main_loop_raw(
    surface: &emapi::sdl::Surface,
    numfont: &emapi::sdl::Surface,
    img: &emapi::sdl::Surface,
    se: &emapi::sdl::mixer::Chunk,
) {
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

    let _fps = FPS_STATE.with(|cell| {
        let mut state = cell.borrow_mut();

        let now = emapi::emscripten::performance_now();
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
            log::trace!("fps: {:.1}", state.fps);

            let b = se.play();
            log::info!("play se: {b}");
        }

        state.fps
    });

    // update & render
    main_loop(surface);
    // draw fps
    img.blit(surface).expect("blit failed");
    numfont.blit(surface).expect("blit failed");
    // show as main canvas
    surface.flip().expect("flip failed");
}

fn setup_main_loop() -> anyhow::Result<()> {
    emapi::sdl::init()?;
    let surface = emapi::sdl::set_video_mode(
        640,
        480,
        32,
        emapi::sdl::flags::SDL_SWSURFACE | emapi::sdl::flags::SDL_DOUBLEBUF,
    )?;

    emapi::sdl::ttf::init()?;
    emapi::sdl::image::init()?;
    emapi::sdl::mixer::init()?;
    emapi::sdl::mixer::open_audio()?;

    let font = emapi::sdl::ttf::open_font("monospace", 16)?;
    let numfont = font.render("hello", emapi::sdl::Color::WHITE)?;

    let img = emapi::sdl::image::load_from_memory(super::res::SAMPLE_IMG)?;
    log::info!("load img OK");

    let se = emapi::sdl::mixer::load_from_memory(super::res::SAMPLE_SE)?;
    log::info!("load se OK");

    // fps (not 0) does not work well
    // probably because of security issue?
    // fps=0 means to use requestAnimationFrame()
    emapi::emscripten::set_main_loop(0, move || {
        main_loop_raw(&surface, &numfont, &img, &se);
    });

    Ok(())
}
