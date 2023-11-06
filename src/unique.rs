//!Unique pointer implementation

use core::{mem, fmt, ptr, marker};

use crate::Deleter;

#[cfg(feature = "alloc")]
///Alias to `Unique` with `GlobalDeleter` as second type parameter
pub type Global<T> = Unique<'static, T, crate::GlobalDeleter>;

#[cfg(feature = "alloc")]
impl<T> Global<T> {
    #[inline]
    ///Creates new instance using global allocator
    pub fn boxed(val: T) -> Self {
        alloc::boxed::Box::new(val).into()
    }
}

#[cfg(feature = "alloc")]
impl<T: ?Sized> Global<T> {
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
///
///# Trait implementation
///
///Due to assumption that `Unique` pointer always has valid value
///(you need to use unsafe code to create invalid one)
///
///All trait implementations, except pointer specific one (e.g. `fmt::Pointer`), implements
///corresponding traits by delegating call to underlying value.
pub struct Unique<'a, T: ?Sized, D: Deleter> {
    inner: ptr::NonNull<T>,
    _traits: marker::PhantomData<&'a D>,
}

impl<'a, T: ?Sized, D: Deleter> Unique<'a, T, D> {
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
    ///
    ///Note that it is illegal to create multiple mutable references
    ///so care must be taken when converting raw pointer into mutable reference.
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

impl<'a, T: ?Sized, D: Deleter> Drop for Unique<'a, T, D> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            D::delete::<T>(self.inner.as_ptr())
        }
    }
}

impl<'a, T: ?Sized, D: Deleter> fmt::Pointer for Unique<'a, T, D> {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.inner, fmt)
    }
}

impl<'a, T: ?Sized + fmt::Debug, D: Deleter> fmt::Debug for Unique<'a, T, D> {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.as_ref(), fmt)
    }
}

impl<'a, T: ?Sized + fmt::Display, D: Deleter> fmt::Display for Unique<'a, T, D> {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_ref(), fmt)
    }
}

impl<'a, T: ?Sized + Unpin, D: Deleter> Unpin for Unique<'a, T, D> {}

unsafe impl<'a, T: ?Sized + Send, D: Deleter> Send for Unique<'a, T, D> {}

unsafe impl<'a, T: ?Sized + Sync, D: Deleter> Sync for Unique<'a, T, D> {}

impl<'a, T: ?Sized, D: Deleter> core::ops::Deref for Unique<'a, T, D> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.inner.as_ptr()
        }
    }
}

impl<'a, T: ?Sized, D: Deleter> core::ops::DerefMut for Unique<'a, T, D> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.inner.as_ptr()
        }
    }
}

impl<'a, T: ?Sized + core::hash::Hash, D: Deleter> core::hash::Hash for Unique<'a, T, D> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl<'a, T: ?Sized + PartialOrd, D: Deleter> PartialOrd<Self> for Unique<'a, T, D> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(self.as_ref(), other.as_ref())
    }
}

impl<'a, T: ?Sized + PartialEq, D: Deleter> PartialEq<Self> for Unique<'a, T, D> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.as_ref(), other.as_ref())
    }

    #[allow(clippy::partialeq_ne_impl)]
    #[inline(always)]
    fn ne(&self, other: &Self) -> bool {
        PartialEq::ne(self.as_ref(), other.as_ref())
    }
}

impl<'a, T: ?Sized + Eq, D: Deleter> Eq for Unique<'a, T, D> {
}

impl<'a, T: ?Sized + core::panic::RefUnwindSafe, D: Deleter> core::panic::UnwindSafe for Unique<'a, T, D> {
}

impl<'a, T: ?Sized + core::panic::RefUnwindSafe, D: Deleter> core::panic::RefUnwindSafe for Unique<'a, T, D> {
}

#[cfg(feature = "alloc")]
impl<T: ?Sized> From<alloc::boxed::Box<T>> for Global<T> {
    #[inline]
    fn from(ptr: alloc::boxed::Box<T>) -> Self {
        let ptr = alloc::boxed::Box::into_raw(ptr);
        unsafe {
            Self::from_ptr_unchecked(ptr)
        }
    }
}

#[cfg(feature = "alloc")]
impl<T: ?Sized + Clone> Clone for Global<T> {
    fn clone(&self) -> Self {
        let val = unsafe {
            //Make sure not to drop unnecessary
            core::mem::ManuallyDrop::new(
                alloc::boxed::Box::from_raw(self.get())
            )
        };
        let result = core::mem::ManuallyDrop::into_inner(val.clone());
        result.into()
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for Unique<'a, T, ()> {
    #[inline]
    fn from(ptr: &'a mut T) -> Self {
        unsafe {
            Self::from_ptr_unchecked(ptr)
        }
    }
}
