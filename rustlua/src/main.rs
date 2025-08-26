mod emscripten;

use mlua::{Lua, LuaOptions, StdLib};

use crate::emscripten::setup_logger;

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
function f() {
    if (!document) return null;
    var e = document.getElementById('source');
    if (!e) return null;
    return e.value;
}
f()
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

fn main_loop_raw() {
    log::trace!("frame");
}

fn setup_main_loop() {
    emscripten::set_main_loop(1, main_loop_raw);
}

fn main() -> anyhow::Result<()> {
    setup_logger(log::LevelFilter::Trace);

    print_test();
    set_callback_button_clicked();
    if let Err(err) = lua_test() {
        eprintln!("{err}");
    }

    setup_main_loop();

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
    Ok(())
}
