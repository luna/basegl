//! Generic color management implementation. Implements multiple color spaces, including `Rgb`,
//! `LinearRgb`, `Hsv`, `Hsl`, `Xyz`, `Lab`, `Lch`, and others. Provides conversion utilities and
//! many helpers. It is inspired by different libraries, including Rust Palette. We are not using
//! Palette here because it is buggy (https://github.com/Ogeon/palette/issues/187), uses bounds
//! on structs which makes the bound appear in places they should not, uses too strict bounds,
//! and does not provide many useful conversions. Moreover, this library is not so generic, uses
//! `f32` everywhere and is much simpler.
//!
//! **WARNING**
//! Be extra careful when developing color conversion equations. Many equations were re-scaled to
//! make them more pleasant to work, however, the equations you will fnd will probably work on
//! different value ranges. Read documentation for each color space very carefully.

pub mod component;
pub mod data;
pub mod gradient;
pub mod space;

pub use component::*;
pub use self::data::*;
pub use gradient::*;
pub use space::*;
