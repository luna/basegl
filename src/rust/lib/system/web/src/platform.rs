//! This module provides helpers for platform specific logic.

use std::convert::TryFrom;



// ================
// === Platform ===
// ================

/// This enumeration lists all the supported platforms.
#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Platform {
    Android,
    FreeBSD,
    IOS,
    Linux,
    MacOS,
    OpenBSD,
    Windows,
}
pub use Platform::*;

#[derive(Clone,Copy,Debug)]
pub struct UnknownPlatform;

impl TryFrom<&str> for Platform {
    type Error = UnknownPlatform;
    #[allow(clippy::if_same_then_else)]
    fn try_from(s:&str) -> Result<Self,Self::Error> {
        let name = s.to_lowercase();
        if      name.find("android").is_some() { Ok(Android) }
        else if name.find("freebsd").is_some() { Ok(FreeBSD) }
        else if name.find("openbsd").is_some() { Ok(OpenBSD) }
        else if name.find("bsd").is_some()     { Ok(FreeBSD) }
        else if name.find("ios").is_some()     { Ok(IOS) }
        else if name.find("iphone").is_some()  { Ok(IOS) }
        else if name.find("ipad").is_some()    { Ok(IOS) }
        else if name.find("linux").is_some()   { Ok(Linux) }
        else if name.find("mac").is_some()     { Ok(MacOS) }
        else if name.find("darwin").is_some()  { Ok(MacOS) }
        else if name.find("win").is_some()     { Ok(Windows) }
        else                                   { Err(UnknownPlatform) }
    }
}

impl TryFrom<String> for Platform {
    type Error = UnknownPlatform;
    fn try_from(s:String) -> Result<Self,Self::Error> {
        Platform::try_from(s.as_str())
    }
}



// ================================
// === Compile Time Redirection ===
// ================================

/// Queries which platform we are on.
#[cfg(target_arch="wasm32")]
pub fn current() -> Platform {
    current_wasm()
}

/// Queries which platform we are on.
#[cfg(not(target_arch="wasm32"))]
pub fn current() -> Platform {
    current_native()
}



// ====================
// === Current WASM ===
// ====================

/// Queries which platform we are on.
#[allow(clippy::if_same_then_else)]
pub fn current_wasm() -> Option<Platform> {
    use super::window;
    let window    = window();
    let navigator = window.navigator();
    let platform  = navigator.platform().unwrap_or_default().to_lowercase();
    let agent     = navigator.user_agent().unwrap_or_default().to_lowercase();
    Platform::try_from(platform).ok().or_else(||Platform::try_from(agent).ok())
}



// ======================
// === Current Native ===
// ======================

#[cfg(target_os="android")] fn current_native() -> Platform { Android }
#[cfg(target_os="ios")]     fn current_native() -> Platform { IOS }
#[cfg(target_os="linux")]   fn current_native() -> Platform { Linux }
#[cfg(target_os="macos")]   fn current_native() -> Platform { MacOS }
#[cfg(target_os="windows")] fn current_native() -> Platform { Windows }

#[cfg(not(any(
    target_arch = "wasm32",
    target_os   = "android",
    target_os   = "ios",
    target_os   = "linux",
    target_os   = "macos",
    target_os   = "windows"
)))] fn current_native() -> Platform { Unknown }



// =============
// === Tests ===
// =============

#[cfg(all(test,any(target_os="linux",target_os="windows",target_os="macos")))]
mod test {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn platform() {
        assert_eq!(current(),target_os())
    }
}
