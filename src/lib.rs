//!Smart pointers for Rust
//!
//!## Features
//!
//!- `alloc` Enables usage of `alloc` crate

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

#[cfg(feature = "alloc")]
extern crate alloc;

///Describes how to de-allocate pointer.
pub trait Deleter {
    ///This function is called on `Drop`
    fn delete<T>(&mut self, ptr: *mut u8);
}

impl Deleter for () {
    fn delete<T>(&mut self, _: *mut u8) {}
}

impl Deleter for unsafe extern "C" fn(*mut u8) {
    fn delete<T>(&mut self, ptr: *mut u8) {
        unsafe {
            (*self)(ptr)
        }
    }
}

impl Deleter for unsafe fn(*mut u8) {
    fn delete<T>(&mut self, ptr: *mut u8) {
        unsafe {
            (*self)(ptr)
        }
    }
}

impl<F: FnMut(*mut u8)> Deleter for F {
    fn delete<T>(&mut self, ptr: *mut u8) {
        (*self)(ptr)
    }
}

#[cfg(feature = "alloc")]
///Default Rust deleter.
///
///Invokes destructor and de-allocates memory using global allocator, using Box.
///
///It can be useful when one would want to allow type erasure,
///but it is UB to specify invalid `T`
pub fn default_deleter<T>(ptr: *mut u8) {
    unsafe {
        alloc::boxed::Box::from_raw(ptr as *mut T);
    }
}

#[derive(Default)]
///Deleter which uses global allocator.
///
///It uses type information, provided as type parameter of `Deleter::delete` to create layout for `alloc::dealloc`
///
///Therefore user must guarantee that pointer was created with the same type information
pub struct DefaultDeleter;

#[cfg(feature = "alloc")]
impl Deleter for DefaultDeleter {
    #[inline]
    fn delete<T>(&mut self, ptr: *mut u8) {
        default_deleter::<T>(ptr)
    }
}

pub mod utils;
pub mod unique;
pub use unique::Unique;
