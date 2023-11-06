use smart_ptr::{unique, Unique};

use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

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
        let _ptr = unsafe { unique::Unique::<_, ()>::new(&mut value) };
    }

    assert!(is_drop);
}

#[test]
fn should_dealloc() {
    static IS_DEALLOC: AtomicBool = AtomicBool::new(false);
    pub struct MyDeleter<'a>(&'a mut bool);

    impl<'a> smart_ptr::Deleter for MyDeleter<'a> {
        unsafe fn delete<T>(_: *mut ()) {
            IS_DEALLOC.store(true, Ordering::SeqCst);
        }
    }

    {
        let mut value = false;
        let _ptr = unsafe { Unique::<bool, MyDeleter>::new(&mut value) };
    }

    assert!(IS_DEALLOC.load(Ordering::SeqCst));
}

#[test]
#[should_panic]
fn should_panic_on_null() {
    unsafe {
        unique::Unique::<bool, ()>::new(ptr::null_mut());
    }
}

#[test]
fn should_fail_on_null() {
    unsafe {
        assert!(unique::Unique::<bool, ()>::from_ptr(ptr::null_mut()).is_none());
    }
}

#[test]
fn should_handle_mut_ref() {
    let mut test = false;
    let mut ptr: Unique::<bool, ()> = (&mut test).into();
    *ptr = true;
    drop(ptr);
    assert!(test);
}
