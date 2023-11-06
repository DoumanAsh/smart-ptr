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
    unsafe fn delete<T: ?Sized>(ptr: *mut T);
}

impl Deleter for () {
    #[inline(always)]
    unsafe fn delete<T: ?Sized>(_: *mut T) {}
}

#[cfg(feature = "alloc")]
///Default Rust deleter.
///
///Invokes destructor and de-allocates memory using global allocator, using Box.
///
///It can be useful when one would want to allow type erasure,
///but it is UB to specify invalid `T`
///
///```rust
///use smart_ptr::Unique;
///
///let var = Box::new("test".to_string());
///unsafe {
///    smart_ptr::boxed_deleter::<String>(Box::leak(var) as *mut String);
///}
///```
///
///## Warning
///
///Remember that things can get complicated when you cast from fat ptrs(with vtable)
pub unsafe fn boxed_deleter<T: ?Sized>(ptr: *mut T) {
    debug_assert!(!ptr.is_null());

    let _  = alloc::boxed::Box::from_raw(ptr);
}

#[derive(Default)]
///Deleter which uses global allocator via `Box`.
///
///It uses type information, provided as type parameter of `Deleter::delete` to re-create `Box` and
///destruct it
///
///Therefore user must guarantee that pointer was created with the same type information
pub struct GlobalDeleter;

#[cfg(feature = "alloc")]
impl Deleter for GlobalDeleter {
    #[inline]
    unsafe fn delete<T: ?Sized>(ptr: *mut T) {
        boxed_deleter::<T>(ptr)
    }
}

pub mod unique;
pub use unique::Unique;
