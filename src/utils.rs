//! Utilities module

///Simple guard structure that calls closure on Drop
pub struct CallOnDrop<F: FnMut()>(pub F);

impl<F: FnMut()> Drop for CallOnDrop<F> {
    fn drop(&mut self) {
        (self.0)()
    }
}
