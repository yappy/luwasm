use std::ffi::CStr;

mod ffi {
    #![allow(dead_code)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/bind_sdl.rs"));
}

/// Nesessarily return [Err] with SDL_GetError() message.
fn sdl_error() -> anyhow::Result<()> {
    let p = unsafe { ffi::SDL_GetError() };
    if !p.is_null() {
        let s = unsafe { CStr::from_ptr(p) };
        let msg = s.to_str().unwrap();
        anyhow::bail!("SDL error: {msg}")
    } else {
        anyhow::bail!("Unknown SDL error")
    }
}

pub mod init {
    #![allow(unused_imports)]
    pub use super::ffi::SDL_INIT_AUDIO;
    pub use super::ffi::SDL_INIT_EVERYTHING;
    pub use super::ffi::SDL_INIT_HAPTIC;
    pub use super::ffi::SDL_INIT_JOYSTICK;
    pub use super::ffi::SDL_INIT_NOPARACHUTE;
    pub use super::ffi::SDL_INIT_TIMER;
    pub use super::ffi::SDL_INIT_VIDEO;
}

pub fn init(flags: i32) -> anyhow::Result<()> {
    let ret = unsafe { ffi::SDL_Init(flags as u32) };
    if ret < 0 {
        sdl_error()?;
    }
    Ok(())
}
