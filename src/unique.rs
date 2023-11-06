//!Unique pointer implementation

use core::{mem, fmt, ptr, marker};

use crate::Deleter;

#[cfg(feature = "alloc")]
///Alias to `Unique` with `DefaultDeleter` as second type parameter
pub type Global<T> = Unique<'static, T, crate::DefaultDeleter>;

#[cfg(feature = "alloc")]
impl<T> Global<T> {
    #[inline]
    ///Creates new instance using global allocator
    pub fn boxed(val: T) -> Self {
        alloc::boxed::Box::new(val).into()
    }

    #[inline]
    ///Converts ptr to box
    pub fn into_boxed(self) -> alloc::boxed::Box<T> {
        let ptr = self.release().as_ptr();
        unsafe {
            alloc::boxed::Box::from_raw(ptr)
        }
    }
}

#[repr(transparent)]
///Smart pointer, that owns and manages object via its pointer.
///
///On `Drop` it automatically disposes of pointer with provided deleter.
///
///Useful in C FFI context.
///
///# Safety
///
///If you use [Deleter](trait.Deleter.html) that relies on type information, you must guarantee
///that object was created using the same type as pointer, which points to it.
///
///Which means you must guarantee that specified pointer is valid one and points to existing memory storage,
///which is already initialized.
pub struct Unique<'a, T, D> where D: Deleter {
    inner: ptr::NonNull<T>,
    _traits: marker::PhantomData<&'a D>,
}

impl<'a, T, D: Deleter> Unique<'a, T, D> {
    #[inline]
    ///Creates new instance from raw pointer and `Deleter` instance
    ///
    ///# Panics
    ///
    ///- If pointer is null
    pub unsafe fn new(ptr: *mut T) -> Self {
        assert!(!ptr.is_null());

        Self::from_ptr_unchecked(ptr)
    }

    #[inline]
    ///Creates instance from raw pointer, checking if pointer is null.
    ///
    ///Returns `None` if pointer is null.
    pub unsafe fn from_ptr(ptr: *mut T) -> Option<Self> {
        match ptr.is_null() {
            true => None,
            false => Some(Self::from_ptr_unchecked(ptr)),
        }
    }

    #[inline]
    ///Creates instance from raw pointer, without checking if pointer is null.
    ///
    ///User must ensure that pointer is non-null
    pub unsafe fn from_ptr_unchecked(ptr: *mut T) -> Self {
        Self {
            inner: ptr::NonNull::new_unchecked(ptr),
            _traits: marker::PhantomData,
        }
    }

    #[inline(always)]
    ///Gets underlying raw pointer.
    pub fn get(&self) -> *mut T {
        self.inner.as_ptr()
    }

    #[inline(always)]
    ///Gets reference to underlying data.
    pub fn as_ref(&self) -> &T {
        self
    }

    #[inline(always)]
    ///Gets mutable reference to underlying data.
    pub fn as_mut(&mut self) -> &mut T {
        self
    }

    #[inline(always)]
    ///Retrieves pointer as of type
    pub fn cast<N>(&self) -> *mut N {
        self.inner.as_ptr() as *mut N
    }

    #[inline(always)]
    ///Retrieves pointer as of type and const
    pub fn const_cast<N>(&self) -> *const N {
        self.inner.as_ptr() as *mut N as *const N
    }

    #[inline(always)]
    ///Swaps underlying pointers between instances
    pub fn swap(&mut self, other: &mut Self) {
        mem::swap(&mut self.inner, &mut other.inner);
    }

    #[inline]
    ///Releases the ownership and returns raw pointer, without dropping it.
    pub fn release(self) -> ptr::NonNull<T> {
        let result = self.inner;
        mem::forget(self);
        result
    }
}

impl<'a, T, D: Deleter> Drop for Unique<'a, T, D> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            D::delete::<T>(self.inner.as_ptr() as *mut ())
        }
    }
}

impl<'a, T, D: Deleter> fmt::Pointer for Unique<'a, T, D> {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.inner, fmt)
    }
}

impl<'a, T, D: Deleter> fmt::Debug for Unique<'a, T, D> {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, fmt)
    }
}

impl<'a, T: Unpin, D: Deleter> Unpin for Unique<'a, T, D> {}

unsafe impl<'a, T: Send, D: Deleter> Send for Unique<'a, T, D> {}

unsafe impl<'a, T: Sync, D: Deleter> Sync for Unique<'a, T, D> {}

impl<'a, T, D: Deleter> core::ops::Deref for Unique<'a, T, D> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.inner.as_ptr()
        }
    }
}

impl<'a, T, D: Deleter> core::ops::DerefMut for Unique<'a, T, D> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.inner.as_ptr()
        }
    }
}

impl<'a, T, D: Deleter> core::hash::Hash for Unique<'a, T, D> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

#[cfg(feature = "alloc")]
impl<T> From<alloc::boxed::Box<T>> for Global<T> {
    #[inline]
    fn from(ptr: alloc::boxed::Box<T>) -> Self {
        let ptr = alloc::boxed::Box::into_raw(ptr);
        unsafe {
            Self::from_ptr_unchecked(ptr)
        }
    }
}

impl<'a, T> From<&'a mut T> for Unique<'a, T, ()> {
    #[inline]
    fn from(ptr: &'a mut T) -> Self {
        unsafe {
            Self::from_ptr_unchecked(ptr)
        }
    }
}
