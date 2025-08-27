use std::{cell::RefCell, ffi::CStr, ffi::CString};

mod ffi {
    #![allow(dead_code)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/bind_emscripten.rs"));
}

pub enum LogTarget {
    // C stdout/stderr
    ModuleOut,
    ModuleErr,
    // JS console
    ConsoleError,
    ConsoleWarn,
    ConsoleInfo,
    ConsoleDebug,
}

pub fn log(target: LogTarget, msg: &str) {
    let flags = match target {
        LogTarget::ModuleOut => ffi::EM_LOG_INFO,
        LogTarget::ModuleErr => ffi::EM_LOG_ERROR,
        LogTarget::ConsoleError => ffi::EM_LOG_CONSOLE | ffi::EM_LOG_ERROR,
        LogTarget::ConsoleWarn => ffi::EM_LOG_CONSOLE | ffi::EM_LOG_WARN,
        LogTarget::ConsoleInfo => ffi::EM_LOG_CONSOLE | ffi::EM_LOG_INFO,
        LogTarget::ConsoleDebug => ffi::EM_LOG_CONSOLE | ffi::EM_LOG_DEBUG,
    };
    let msg = CString::new(msg).unwrap();

    unsafe {
        ffi::emscripten_log(flags as i32, c"%s".as_ptr(), msg.as_ptr());
    }
}

struct EmLogger;

impl log::Log for EmLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let target = match record.level() {
                log::Level::Error => LogTarget::ConsoleError,
                log::Level::Warn => LogTarget::ConsoleWarn,
                log::Level::Info => LogTarget::ConsoleInfo,
                log::Level::Debug => LogTarget::ConsoleDebug,
                log::Level::Trace => LogTarget::ConsoleDebug,
            };
            self::log(target, &format!("{}", record.args()));
        }
    }

    fn flush(&self) {}
}

/// Dispatch info!() etc. to [emscripten::log()].
pub fn setup_logger(level: log::LevelFilter) {
    static LOGGER: EmLogger = EmLogger;

    log::set_logger(&LOGGER).expect("set_logger failed");
    log::set_max_level(level);
}

/// Warning: if js code throws exception, it causes undefined behavior.
pub fn eval_js(src: &str) -> Option<String> {
    let src = CString::new(src).unwrap();
    let p = unsafe { ffi::emscripten_run_script_string(src.as_ptr()) };
    if !p.is_null() {
        let s = unsafe { CStr::from_ptr(p) };
        Some(s.to_str().unwrap().to_string())
    } else {
        None
    }
}

/// Calls JavaScript `performance.now()` function. (ms)
pub fn performance_now() -> f64 {
    unsafe { ffi::emscripten_performance_now() }
}

/// <https://emscripten.org/docs/api_reference/emscripten.h.html#c.emscripten_set_main_loop>
pub fn set_main_loop<F>(fps: i32, func: F)
where
    F: FnMut() + 'static,
{
    thread_local! {
        // global Option<FnMut()->()>
        static MAIN_LOOP_FUNC: RefCell<Option<Box<dyn FnMut()>>> = RefCell::new(None);
    }
    extern "C" fn main_loop_func() {
        MAIN_LOOP_FUNC.with(|cell| {
            let mut func = cell.borrow_mut();
            if let Some(f) = func.as_mut() {
                f();
            }
        });
    }

    MAIN_LOOP_FUNC.with(|cell| {
        let mut f = cell.borrow_mut();
        assert!(f.is_none(), "set_main_loop() called twice");
        *f = Some(Box::new(func));
    });
    unsafe {
        // If simulate_infinite_loop is true, this function will throw
        // an exception. Set false because it is dangerous.
        ffi::emscripten_set_main_loop(Some(main_loop_func), fps, false);
    }
}

pub use ffi::EmscriptenMouseEvent as MouseEvent;

/// Warning: handler cannot be deleted. Do not call repeatedly.
/// * `target`: CSS selector like `#id`
pub fn set_click_callback<F>(target: &str, func: F) -> anyhow::Result<usize>
where
    F: FnMut(i32, &MouseEvent) -> bool + 'static,
{
    type MouseCallback = Box<dyn FnMut(i32, &MouseEvent) -> bool>;
    thread_local! {
        static MOUSE_HANDLERS: RefCell<Vec<MouseCallback>> = RefCell::new(Vec::new());
    }

    extern "C" fn callback(
        event_type: ::std::os::raw::c_int,
        mouse_event: *const ffi::EmscriptenMouseEvent,
        user_data: *mut ::std::os::raw::c_void,
    ) -> bool {
        // void* to usize
        let id = user_data as usize;
        // C const pointer to Rust ref
        let event = unsafe { &*mouse_event };
        MOUSE_HANDLERS.with(|cell| {
            let mut v = cell.borrow_mut();
            v[id](event_type, event)
        })
    }

    let target = CString::new(target).unwrap();
    let id = MOUSE_HANDLERS.with(|cell| {
        let mut v = cell.borrow_mut();
        v.push(Box::new(func));
        v.len() - 1
    });

    // #define EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD ((pthread_t)0x2)
    const EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD: usize = 2;
    let ret = unsafe {
        ffi::emscripten_set_click_callback_on_thread(
            target.as_ptr(),
            id as _,
            false,
            Some(callback),
            EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD as _,
        )
    };
    if ret >= 0 {
        Ok(id)
    } else {
        anyhow::bail!("emscripten_set_click_callback_on_thread: {}", ret);
    }
}
