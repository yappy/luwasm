mod emscripten;

use anyhow::Result;
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
}

fn lua_test() -> Result<()> {
    let libs = StdLib::ALL_SAFE;
    let options = LuaOptions::new().catch_rust_panics(true);
    let lua = Lua::new_with(libs, options)?;

    let chunk = lua.load("print('Hello Lua!')");
    chunk.exec()?;

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
    let chunk = lua.load("this is not lua source").set_name("=<string>");
    // compile and execute
    chunk.exec()?;

    Ok(())
}

fn main() -> Result<()> {
    print_test();
    lua_test()?;

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
