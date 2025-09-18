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

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    #[allow(dead_code)]
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    #[allow(dead_code)]
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
    };

    fn to_sdl_color(&self) -> ffi::SDL_Color {
        ffi::SDL_Color {
            r: self.r,
            g: self.g,
            b: self.b,
            unused: 0,
        }
    }
}

pub fn init() -> anyhow::Result<()> {
    let ret = unsafe { ffi::SDL_Init(ffi::SDL_INIT_EVERYTHING) };
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
#[derive(Clone)]
pub struct Surface(*mut ffi::SDL_Surface);

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            ffi::SDL_FreeSurface(self.0);
        }
    }
}

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

    pub fn blit(&self, dst: &Self) -> anyhow::Result<()> {
        // #define SDL_BlitSurface SDL_UpperBlit
        let ret =
            unsafe { ffi::SDL_UpperBlit(self.0, std::ptr::null(), dst.0, std::ptr::null_mut()) };
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

// -----------------------------------------------------------------------------

pub mod ttf {
    use super::ffi;
    use std::ffi::CString;

    // #define TTF_GetError SDL_GetError
    use super::sdl_error as ttf_error;

    pub fn init() -> anyhow::Result<()> {
        let ret = unsafe { ffi::TTF_Init() };
        if ret < 0 {
            ttf_error()?;
        }

        Ok(())
    }

    pub struct Font(*mut ffi::TTF_Font);

    impl Drop for Font {
        fn drop(&mut self) {
            unsafe {
                ffi::TTF_CloseFont(self.0);
            }
        }
    }

    impl Font {
        pub fn render(&self, text: &str, fg: super::Color) -> anyhow::Result<super::Surface> {
            let text = CString::new(text).unwrap();
            let surface =
                unsafe { ffi::TTF_RenderText_Blended(self.0, text.as_ptr(), fg.to_sdl_color()) };
            if surface.is_null() {
                ttf_error()?;
            }

            Ok(super::Surface(surface))
        }
    }

    pub fn open_font(file: &str, ptsize: i32) -> anyhow::Result<Font> {
        let file = CString::new(file).unwrap();
        let font = unsafe { ffi::TTF_OpenFont(file.as_ptr(), ptsize) };
        if font.is_null() {
            ttf_error()?;
        }

        Ok(Font(font))
    }
}

// -----------------------------------------------------------------------------

pub mod image {
    use anyhow::Context;

    use super::ffi;
    use std::ffi::CString;

    // #define IMG_GetError SDL_GetError
    use super::sdl_error as img_error;

    pub fn init() -> anyhow::Result<()> {
        // IMG_Init will do nothing and flags will be returned as is
        let flags = ffi::IMG_InitFlags_IMG_INIT_JPG | ffi::IMG_InitFlags_IMG_INIT_PNG;
        let flags = flags as i32;
        let ret = unsafe { ffi::IMG_Init(flags) };
        if ret != flags {
            img_error().context("IMG_Init failed")?;
        }

        Ok(())
    }

    pub fn load(file: &str) -> anyhow::Result<super::Surface> {
        let file = CString::new(file).unwrap();
        let sf = unsafe { ffi::IMG_Load(file.as_ptr()) };
        if sf.is_null() {
            img_error().context("IMG_Load failed")?;
        }

        Ok(super::Surface(sf))
    }

    pub fn load_from_memory(bin: &[u8]) -> anyhow::Result<super::Surface> {
        const TMP_PATH: &str = "/tmp/imgload.bin";
        // JS/WASM is single threaded
        // It doesn't cause race conditions
        std::fs::write(TMP_PATH, bin)?;

        load(TMP_PATH)
    }
}

pub mod mixer {
    use anyhow::Context;

    use super::ffi;
    use std::ffi::CString;

    // #define IMG_GetError SDL_GetError
    use super::sdl_error as mixer_error;

    pub fn init() -> anyhow::Result<()> {
        // Mix_Init will do nothing and return MIX_INIT_OGG
        let flags = ffi::MIX_InitFlags_MIX_INIT_OGG;
        let flags = flags as i32;
        let ret = unsafe { ffi::Mix_Init(flags) };
        if ret != flags {
            mixer_error().context("Mix_Init failed")?;
        }

        Ok(())
    }

    pub fn open_audio() -> anyhow::Result<()> {
        let ret = unsafe {
            ffi::Mix_OpenAudio(
                ffi::MIX_DEFAULT_FREQUENCY as i32,
                ffi::MIX_DEFAULT_FORMAT as u16,
                ffi::MIX_DEFAULT_CHANNELS as i32,
                1024, // default value in generated js code
            )
        };
        if ret < 0 {
            mixer_error().context("Mix_OpenAudio failed")?;
        }

        Ok(())
    }

    pub struct Chunk(*mut ffi::Mix_Chunk);

    impl Drop for Chunk {
        fn drop(&mut self) {
            unsafe {
                ffi::Mix_FreeChunk(self.0);
            }
        }
    }

    impl Chunk {
        pub fn play(&self) -> bool {
            let ret = unsafe { ffi::Mix_PlayChannelTimed(-1, self.0, 1, -1) };

            ret >= 0
        }
    }

    pub fn load(file: &str) -> anyhow::Result<Chunk> {
        let file = CString::new(file).unwrap();
        let chunk = unsafe { ffi::Mix_LoadWAV(file.as_ptr()) };
        if chunk.is_null() {
            mixer_error().context("Mix_LoadWAV failed")?;
        }

        Ok(Chunk(chunk))
    }

    pub fn load_from_memory(bin: &[u8]) -> anyhow::Result<Chunk> {
        const TMP_PATH: &str = "/tmp/soundload.bin";
        // JS/WASM is single threaded
        // It doesn't cause race conditions
        std::fs::write(TMP_PATH, bin)?;

        load(TMP_PATH)
    }
}
