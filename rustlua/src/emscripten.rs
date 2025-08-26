use std::{cell::RefCell, ffi::CString};

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
        ffi::emscripten_log(flags, c"%s".as_ptr(), msg.as_ptr());
    }
}

/// Warning: if js code throws exception, it causes undefined behavior.
pub fn eval_js(src: &str) -> Option<String> {
    let src = CString::new(src).unwrap();
    let p = unsafe { ffi::emscripten_run_script_string(src.as_ptr()) };
    if !p.is_null() {
        let s = unsafe { ::std::ffi::CStr::from_ptr(p) };
        Some(s.to_str().unwrap().to_string())
    } else {
        None
    }
}

pub use ffi::EmscriptenMouseEvent as MouseEvent;

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
    let boxed_func: MouseCallback = Box::new(func);
    let id = MOUSE_HANDLERS.with(|cell| {
        let mut v = cell.borrow_mut();
        v.push(boxed_func);
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
        anyhow::bail!("EMSCRIPTEN_RESULT: {}", ret);
    }
}
