use smart_ptr::{unique, Unique};

use core::ptr;

#[test]
fn should_drop_without_dealloc() {
    let mut is_drop = false;

    struct PanicOnDrop<'a>(&'a mut bool);

    impl<'a> Drop for PanicOnDrop<'a> {
        fn drop(&mut self) {
            *(self.0) = true;
        }
    }

    {
        let mut value = PanicOnDrop(&mut is_drop);
        let _ptr = unsafe { unique::NonMem::new_default(&mut value) };
    }

    assert!(is_drop);
}

#[test]
fn should_dealloc() {
    let mut is_dealloc = false;
    pub struct MyDeleter<'a>(&'a mut bool);

    impl<'a> smart_ptr::Deleter for MyDeleter<'a> {
        fn delete<T>(&mut self, _: *mut u8) {
            *(self.0) = true;
        }
    }

    {
        let mut value = false;
        let _ptr = unsafe { Unique::<bool, MyDeleter>::new(&mut value, MyDeleter(&mut is_dealloc)) };
    }

    assert!(is_dealloc);
}

#[cfg(feature = "alloc")]
#[test]
fn should_dtor_global() {
    let mut is_dealloc = false;
    pub struct MyDeleter<'a>(&'a mut bool);
    impl<'a> Drop for MyDeleter<'a> {
        fn drop(&mut self) {
            *(self.0) = true;
        }
    }

    Unique::boxed(MyDeleter(&mut is_dealloc));

    assert!(is_dealloc);

    // Check type erasure
    is_dealloc = false;

    let var = Box::new(MyDeleter(&mut is_dealloc));
    unsafe {
        Unique::new(Box::leak(var) as *mut MyDeleter as *mut u8, smart_ptr::boxed_deleter::<MyDeleter>);
    }

    assert!(is_dealloc);
}

#[test]
#[should_panic]
fn should_panic_on_null() {
    unsafe {
        unique::NonMem::<bool>::new_default(ptr::null_mut());
    }
}

#[test]
fn should_fail_on_null() {
    unsafe {
        assert!(unique::NonMem::<bool>::from_ptr_default(ptr::null_mut()).is_none());
    }
}
