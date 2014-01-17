#[crate_type = "rlib"];
#[crate_id="github.com/xsleonard/sdl2_image-rs#sdl2_image:0.1"];
#[desc = "SDL2_image bindings and wrappers"];
#[comment = "SDL2_image bindings and wrappers"];
#[license = "MIT"];

extern mod sdl2;

use std::libc::{c_int, c_char};
use std::ptr;
use std::cast;
use std::io;
use sdl2::surface::Surface;
use sdl2::render::{Texture, Renderer};
use sdl2::get_error;

// Setup linking for all targets.
#[cfg(target_os="macos")]
mod mac {
    #[cfg(mac_framework)]
    #[link(kind="framework", name="SDL2_image")]
    extern {}

    #[cfg(not(mac_framework))]
    #[link(name="SDL2_image")]
    extern {}
}

#[cfg(target_os="win32")]
#[cfg(target_os="linux")]
#[cfg(target_os="freebsd")]
mod others {
    #[link(name="SDL2_image")]
    extern {}
}

mod ffi;

#[deriving(Clone, Eq, IterBytes, ToStr)]
pub enum InitFlag {
    InitJpg = ffi::IMG_INIT_JPG as int,
    InitPng = ffi::IMG_INIT_PNG as int,
    InitTif = ffi::IMG_INIT_TIF as int,
    InitWebp = ffi::IMG_INIT_WEBP as int,
}

pub struct ImageVersion {
    major: int,
    minor: int,
    patch: int,
}

impl ToStr for ImageVersion {
    fn to_str(&self) -> ~str {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl ImageVersion {
    fn from_sdl_version(sv: *ffi::SDL_version) -> ImageVersion {
        unsafe {
            let v = *sv;
            ImageVersion{ major: v.major, minor: v.minor, patch: v.patch }
        }
    }
}


// TODO -- this should be in rust-sdl2
// Most of the sdl2_image API relies on SDL_RWops.

// #[deriving(Eq)]
// pub struct RWops {
//     raw: *SDL_RWops;
//     owned: bool;
// }

// impl Drop for RWops {
//     fn drop(&mut self) {
//         if self.owned {
//             unsafe {
//                 // TODO -- close() returns a c_int error status.
//                 // How do we deal with errors in the destructor?
//                 // Probably either kill the task, or don't implement this
//                 // as a destructor
//                 self.raw.close()
//             }
//         }
//     }
// }

pub trait ImageLoader<T> {
    fn from_file(filename: &str) -> Result<~T, ~str>;
    fn from_xpm_array(xpm: **i8) -> Result<~T, ~str>;
}

pub trait ImageSaver {
    fn save(&self, filename: &str) -> Result<(), ~str>;
}

// TODO -- does this need to be pub'd?
impl ImageLoader<Surface> for Surface {
    fn from_file(filename: &str) -> Result<~Surface, ~str> {
        unsafe {
            let raw = ffi::IMG_Load(filename.to_c_str().unwrap());
            if raw == ptr::null() {
                Err(get_error())
            } else {
                Ok(~Surface { raw: raw, owned: true })
            }
        }
    }

    fn from_xpm_array(xpm: **i8) -> Result<~Surface, ~str> {
        unsafe {
            let raw = ffi::IMG_ReadXPMFromArray(xpm as **c_char);
            if raw == ptr::null() {
                Err(get_error())
            } else {
                Ok(~Surface { raw: raw, owned: true })
            }
        }
    }
}

impl ImageSaver for Surface {
    fn save(&self, filename: &str) -> Result<(), ~str> {
        unsafe {
            let status = ffi::IMG_SavePNG(self.raw,
                                          filename.to_c_str().unwrap());
            if status != 0 {
                Err(get_error())
            } else {
                Ok(())
            }
        }
    }
}

pub trait TextureLoader {
    fn load_texture_from_file(&self, filename: &str) -> Result<~Texture, ~str>;
}

impl TextureLoader for Renderer {
    fn load_texture_from_file(&self,
                              filename: &str) -> Result<~Texture, ~str> {
        unsafe {
            let raw = ffi::IMG_LoadTexture(self.raw,
                                           filename.to_c_str().unwrap());
            if raw == ptr::null() {
                Err(get_error())
            } else {
                Ok(~Texture{ raw: raw, owned: true })
            }
        }
    }
}

pub fn init(flags: &[InitFlag]) -> ~[InitFlag] {
    //! Initializes SDL2_image with InitFlags and returns which
    //! InitFlags were actually used.
    let mut used = ~[];
    unsafe {
        let used_flags = ffi::IMG_Init(
            flags.iter().fold(0, |flags, &flag| {
                flags | flag as ffi::IMG_InitFlags
            })
        );
        for flag in flags.iter() {
            if used_flags & *flag as c_int != 0 {
                used.push(*flag)
            }
        }
    }
    used
}

pub fn quit() {
    unsafe { ffi::IMG_Quit(); }
}

pub fn get_linked_version() -> ImageVersion {
    //! Returns the version of the dynamically linked SDL_image library
    unsafe {
        ImageVersion::from_sdl_version(ffi::IMG_Linked_Version())
    }
}