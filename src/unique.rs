//!Unique pointer implementation

use core::{mem, fmt, ptr};

use crate::Deleter;

///Alias to `Unique` with `()` as second type parameter, which has no deallocation
pub type NonMem<T> = Unique<T, ()>;

#[cfg(feature = "alloc")]
///Alias to `Unique` with `GlobalDeleter` as second type parameter
pub type Global<T> = Unique<T, crate::GlobalDeleter>;

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
///You must guarantee that specified pointer is valid one and points to existing memory storage,
///which is already initialzied.
pub struct Unique<T, D> where D: Deleter {
    inner: ptr::NonNull<T>,
    deleter: D,
}

impl<T, D: Default + Deleter> Unique<T, D> {
    #[inline]
    ///Creates new instance from raw pointer and `Deleter` instance
    ///
    ///# Panics
    ///
    ///- If pointer is null
    pub unsafe fn new_default(ptr: *mut T) -> Self {
        Self::new(ptr, D::default())
    }

    #[inline]
    ///Creates instance from raw pointer, checking if pointer is null.
    ///
    ///Returns `None` if pointer is null.
    pub unsafe fn from_ptr_default(ptr: *mut T) -> Option<Self> {
        Self::from_ptr(ptr, D::default())
    }
}

impl<T, D: Deleter> Unique<T, D> {
    #[inline]
    ///Creates new instance from raw pointer and `Deleter` instance
    ///
    ///# Panics
    ///
    ///- If pointer is null
    pub unsafe fn new(ptr: *mut T, deleter: D) -> Self {
        debug_assert!(!ptr.is_null());

        Self {
            inner: ptr::NonNull::new_unchecked(ptr),
            deleter,
        }
    }

    #[inline]
    ///Creates instance from raw pointer, checking if pointer is null.
    ///
    ///Returns `None` if pointer is null.
    pub unsafe fn from_ptr(ptr: *mut T, deleter: D) -> Option<Self> {
        ptr::NonNull::new(ptr).map(move |inner| Self {
            inner,
            deleter,
        })
    }

    #[inline(always)]
    ///Gets underlying raw pointer.
    pub fn as_raw(&self) -> *mut T {
        self.inner.as_ptr()
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

impl<T, D: Deleter> Drop for Unique<T, D> {
    fn drop(&mut self) {
        let ptr = self.inner.as_ptr();

        let _memory_guard = crate::utils::CallOnDrop(|| self.deleter.delete::<T>(ptr as *mut u8));

        unsafe {
            ptr::drop_in_place(ptr);
        }
    }
}

impl<T, D: Deleter> fmt::Pointer for Unique<T, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.inner)
    }
}

impl<T, D: Deleter> fmt::Debug for Unique<T, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.inner)
    }
}

impl<T: Unpin, D: Deleter + Unpin> Unpin for Unique<T, D> {}

unsafe impl<T: Send, D: Deleter + Send> Send for Unique<T, D> {}

unsafe impl<T: Sync, D: Deleter + Sync> Sync for Unique<T, D> {}

impl<T, D: Deleter> core::ops::Deref for Unique<T, D> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.inner.as_ptr()
        }
    }
}

impl<T, D: Deleter> core::ops::DerefMut for Unique<T, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.inner.as_ptr()
        }
    }
}
