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

pub fn init(flags: u32) -> anyhow::Result<()> {
    let ret = unsafe { ffi::SDL_Init(flags) };
    if ret < 0 {
        sdl_error()?;
    }
    Ok(())
}

pub mod flags {
    #![allow(unused_imports)]
    pub use super::ffi::SDL_ANYFORMAT;
    pub use super::ffi::SDL_ASYNCBLIT;
    pub use super::ffi::SDL_DOUBLEBUF;
    pub use super::ffi::SDL_FULLSCREEN;
    pub use super::ffi::SDL_HWPALETTE;
    pub use super::ffi::SDL_HWSURFACE;
    pub use super::ffi::SDL_NOFRAME;
    pub use super::ffi::SDL_OPENGL;
    pub use super::ffi::SDL_RESIZABLE;
    pub use super::ffi::SDL_SWSURFACE;
}

/// ```C
/// typedef struct SDL_Surface {
///     Uint32 flags;                           /* Read-only */
///     SDL_PixelFormat *format;                /* Read-only */
///     int w, h;                               /* Read-only */
///
///     int pitch;                              /* Read-only */
///     void *pixels;                           /* Read-write */
///
///     /* Application data associated with the surface */
///     void *userdata;                         /* Read-write */
///
///     /* information needed for surfaces requiring locks */
///     int locked;                             /* Read-only */
///     void *lock_data;                        /* Read-only */
///
///     /* clipping information */
///     SDL_Rect clip_rect;                     /* Read-only */
///
///     /* info for fast blit mapping to other surfaces */
///     struct SDL_BlitMap *map;                /* Private */
///
///     /* Reference count -- used when freeing surface */
///     int refcount;                           /* Read-mostly */
///
///     /* This structure also contains private fields not shown here */
/// } SDL_Surface;
/// ```
///
/// The surface returned is freed by SDL_Quit() and
/// should nt be freed by the caller.
#[derive(Clone, Copy)]
pub struct Surface(*mut ffi::SDL_Surface);

#[allow(dead_code)]
impl Surface {
    fn deref(&self) -> &ffi::SDL_Surface {
        unsafe { &*self.0 }
    }

    pub fn x_y_pitch(self) -> (i32, i32, i32) {
        let s = self.deref();
        (s.w, s.h, s.pitch)
    }

    pub fn pixels(&self) -> &mut [u8] {
        let s = self.deref();
        let len = (s.pitch * s.h) as usize;
        // void * => uint8_t *
        let p = s.pixels as *mut u8;

        // C uint8_t* => Rust &[u8]
        unsafe { std::slice::from_raw_parts_mut(p, len) }
    }

    pub const fn must_lock(&self) -> bool {
        true
    }

    pub fn lock(&self) -> anyhow::Result<()> {
        let ret = unsafe { ffi::SDL_LockSurface(self.0) };
        if ret < 0 {
            sdl_error()?;
        }

        Ok(())
    }

    pub fn unlock(&self) {
        unsafe {
            ffi::SDL_UnlockSurface(self.0);
        }
    }

    pub fn flip(&self) -> anyhow::Result<()> {
        let ret = unsafe { ffi::SDL_Flip(self.0) };
        if ret < 0 {
            sdl_error()?;
        }

        Ok(())
    }
}

pub fn set_video_mode(width: i32, height: i32, bpp: i32, flags: u32) -> anyhow::Result<Surface> {
    let surface = unsafe { ffi::SDL_SetVideoMode(width, height, bpp, flags) };
    if surface.is_null() {
        sdl_error()?;
    }

    Ok(Surface(surface))
}
