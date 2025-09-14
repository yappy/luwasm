mod app;
mod emapi;

fn print_test() {
    println!("println to stdout");
    eprintln!("eprintln to stderr");

    emapi::emscripten::log(emapi::emscripten::LogTarget::ModuleOut, "Module stdout");
    emapi::emscripten::log(emapi::emscripten::LogTarget::ModuleErr, "Module stderr");
    emapi::emscripten::log(emapi::emscripten::LogTarget::ConsoleError, "console.error");
    emapi::emscripten::log(emapi::emscripten::LogTarget::ConsoleWarn, "console.warn");
    emapi::emscripten::log(emapi::emscripten::LogTarget::ConsoleInfo, "console.info");
    emapi::emscripten::log(emapi::emscripten::LogTarget::ConsoleDebug, "console.debug");

    log::error!("log::error");
    log::warn!("log::warn");
    log::info!("log::info");
    log::debug!("log::debug");
    log::trace!("log::trace");
}

fn lua_test() -> anyhow::Result<()> {
    app::sys::lua_exec("print('Hello Lua!')")?;
    app::sys::lua_exec("this is not lua source")?;

    Ok(())
}

fn ls(dir: impl AsRef<::std::path::Path>) -> anyhow::Result<()> {
    for (path, etype) in app::fs::ls_recursive(dir, false)? {
        let c = match etype {
            app::fs::EntryType::DIR => 'D',
            app::fs::EntryType::FILE => 'F',
        };
        println!("{c} {}", path.to_str().unwrap());
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

    let dir: &::std::path::Path = "/home/web_user".as_ref();
    let path = dir.join("hello.txt");
    ::std::fs::write(&path, "Hello!\n")?;
    println!("create: hello.txt");

    let json = app::fs::create_fs_image("/home/web_user")?;
    println!("create_fs_image");
    println!("{json}");

    app::fs::import_fs_image(&json, dir)?;
    println!("import_fs_image");

    let content = ::std::fs::read_to_string(&path)?;
    println!("{content}");

    Ok(())
}

fn test_run() -> anyhow::Result<()> {
    print_test();
    if let Err(err) = lua_test() {
        eprintln!("{err}");
    }
    fs_test()?;

    Ok(())
}

fn main() {
    app::jslog::setup_logger(log::LevelFilter::Trace);

    if let Err(err) = app::sys::run() {
        log::error!("{err:#}");
        eprintln!("{err:#}");
    }
    if let Err(err) = test_run() {
        log::error!("{err:#}");
        eprintln!("{err:#}");
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
    log::info!("main exit (keep alive)");
}
