use anyhow::Result;
use mlua::{Lua, LuaOptions, StdLib};

fn main() -> Result<()> {
    println!("Hello, world!");

    let libs = StdLib::ALL_SAFE;
    let options = LuaOptions::new().catch_rust_panics(true);
    let lua = Lua::new_with(libs, options)?;

    let chunk = lua.load("print('Hello Lua!')");
    chunk.exec()?;

    // not compiled yet
    // by default, Rust file:line:column (for panic) will be used for name

    // source: the source of the chunk that created the function.
    // If source starts with a '@', it means that the function was defined
    // in a file where the file name follows the '@'.
    // If source starts with a '=', the remainder of its contents describes
    // the source in a user-dependent manner.
    // Otherwise, the function was defined in a string
    // where source is that string.
    let chunk = lua.load("lexical error").set_name("=<string>");
    // compile and execute
    chunk.exec()?;

    Ok(())
}
