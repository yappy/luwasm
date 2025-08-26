use std::ffi::{ CString};

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
