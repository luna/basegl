use crate::prelude::*;

use crate::backend::webgl::Context;
use crate::tp::debug::TypeDebugName;
use nalgebra::*;
use web_sys::WebGlBuffer;


// =============
// === Types ===
// =============

pub trait MatrixCtx<T,R,C> = where
    T:Scalar, R:DimName, C:DimName,
    nalgebra::DefaultAllocator: nalgebra::allocator::Allocator<T, R, C>;


// =============
// === Empty ===
// =============

/// Trait for types which have empty value.
pub trait Empty {
    fn empty() -> Self;
}

impl Empty for i32 { fn empty() -> Self { 0 } }
impl Empty for f32 { fn empty() -> Self { 0.0 } }
impl<T,R,C> Empty for MatrixMN<T,R,C> where T:Default, Self:MatrixCtx<T,R,C> {
    fn empty() -> Self { Self::repeat(default()) }
}


// ============
// === Item ===
// ============

// === Definition ===

/// Class for buffer items, like `f32` or `Vector<f32>`. It defines utils
/// for mapping the item to WebGL buffer and vice versa.
pub trait Item: Empty {
    type Prim;
    type Dim: DimName;

    /// Count of primitives of the item. For example, `Vector3<f32>` contains
    /// three primitives (`f32` values).
    fn item_count() -> usize {
        <Self::Dim as DimName>::dim()
    }

    /// Conversion from slice of a buffer to the item. Buffers contain primitive
    /// values only, so two `Vector3<f32>` are represented there as six `f32`
    /// values. This allows us to view the buffers using desired types.
    fn from_buffer(buffer: &[Self::Prim]) -> &[Self]
        where Self: std::marker::Sized;

    /// Mutable conversion from slice of a buffer to the item. See the docs for
    /// `from_buffer` to learn more.
    fn from_buffer_mut(buffer: &mut [Self::Prim]) -> &mut [Self]
        where Self: std::marker::Sized;

    fn to_prim_buffer(buffer: &[Self]) -> &[Self::Prim]
        where Self: std::marker::Sized;

    fn to_prim_buffer_mut(buffer: &mut [Self]) -> &mut [Self::Prim]
        where Self: std::marker::Sized;

    /// Creates a JS typed array which is a view into wasm's linear
    /// memory at the slice specified.
    ///
    /// This function returns a new typed array which is a view into
    /// wasm's memory. This view does not copy the underlying data.
    ///
    /// # Unsafety
    ///
    /// Views into WebAssembly memory are only valid so long as the
    /// backing buffer isn't resized in JS. Once this function is called
    /// any future calls to `Box::new` (or malloc of any form) may cause
    /// the returned value here to be invalidated. Use with caution!
    ///
    /// Additionally the returned object can be safely mutated but the
    /// input slice isn't guaranteed to be mutable.
    ///
    /// Finally, the returned object is disconnected from the input
    /// slice's lifetime, so there's no guarantee that the data is read
    /// at the right time.
    unsafe fn js_buffer_view(rust: &[Self::Prim]) -> js_sys::Object;
}

// === Type Families ===

pub type Prim <T> = <T as Item>::Prim;
pub type Dim  <T> = <T as Item>::Dim;

// === Instances ===

impl Item for i32 {
    type Prim = Self;
    type Dim  = U1;

    fn from_buffer     (buffer: &    [Self::Prim]) -> &    [Self] { buffer }
    fn from_buffer_mut (buffer: &mut [Self::Prim]) -> &mut [Self] { buffer }
    fn to_prim_buffer     (buffer: &    [Self]) -> &    [Self::Prim] { buffer }
    fn to_prim_buffer_mut (buffer: &mut [Self]) -> &mut [Self::Prim] { buffer }
    unsafe fn js_buffer_view(data: &[Self::Prim]) -> js_sys::Object {
        js_sys::Int32Array::view(&data).into()
    }
}

impl Item for f32 {
    type Prim = Self;
    type Dim  = U1;

    fn from_buffer     (buffer: &    [Self::Prim]) -> &    [Self] { buffer }
    fn from_buffer_mut (buffer: &mut [Self::Prim]) -> &mut [Self] { buffer }
    fn to_prim_buffer     (buffer: &    [Self]) -> &    [Self::Prim] { buffer }
    fn to_prim_buffer_mut (buffer: &mut [Self]) -> &mut [Self::Prim] { buffer }
    unsafe fn js_buffer_view(data: &[Self::Prim]) -> js_sys::Object {
        js_sys::Float32Array::view(&data).into()
    }
}

impl<T:Item<Prim=T>,R,C> Item for MatrixMN<T,R,C>
    where T:Default, Self:MatrixCtx<T,R,C> {

    type Prim = T;
    type Dim  = R;

    fn from_buffer(buffer: &[Self::Prim]) -> &[Self] {
        // This code casts slice to matrix. This is safe because `MatrixMN`
        // uses `nalgebra::Owned` allocator, which resolves to array defined as
        // `#[repr(C)]` under the hood.
        unsafe {
            let len = buffer.len() / Self::item_count();
            std::slice::from_raw_parts(buffer.as_ptr().cast(), len)
        }
    }

    fn from_buffer_mut(buffer: &mut [Self::Prim]) -> &mut [Self] {
        // This code casts slice to matrix. This is safe because `MatrixMN`
        // uses `nalgebra::Owned` allocator, which resolves to array defined as
        // `#[repr(C)]` under the hood.
        unsafe {
            let len = buffer.len() / Self::item_count();
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr().cast(), len)
        }
    }

    fn to_prim_buffer(buffer: &[Self]) -> &[Self::Prim] {
        // This code casts slice to matrix. This is safe because `MatrixMN`
        // uses `nalgebra::Owned` allocator, which resolves to array defined as
        // `#[repr(C)]` under the hood.
        unsafe {
            let len = buffer.len() * Self::item_count();
            std::slice::from_raw_parts(buffer.as_ptr().cast(), len)
        }
    }

    fn to_prim_buffer_mut(buffer: &mut [Self]) -> &mut [Self::Prim] {
        // This code casts slice to matrix. This is safe because `MatrixMN`
        // uses `nalgebra::Owned` allocator, which resolves to array defined as
        // `#[repr(C)]` under the hood.
        unsafe {
            let len = buffer.len() * Self::item_count();
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr().cast(), len)
        }
    }

    unsafe fn js_buffer_view(data: &[Self::Prim]) -> js_sys::Object {
        <T as Item>::js_buffer_view(data)
    }

}

impl <T,R,C> TypeDebugName for MatrixMN<T,R,C> where Self: MatrixCtx<T,R,C> {
    fn type_debug_name() -> String {
        let col  = <C as DimName>::dim();
        let row  = <R as DimName>::dim();
        let item = type_name::<T>();
        match col {
            1 => format!("Vector{}<{}>"    , row, item),
            _ => format!("Matrix{}x{}<{}>" , row, col, item)
        }
    }
}

