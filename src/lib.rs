#![no_std]

extern crate alloc;

use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt::Debug;

pub mod interface;
pub mod input_pin;
pub mod output_pin;

#[derive(Debug)]
pub struct GenericMock<T: Clone + Debug + PartialEq> {
    expectations: Rc<RefCell<(usize, Vec<T>)>>,
}

impl<'a, T: 'a + Clone + Debug + PartialEq> GenericMock<T> {
    pub fn new(expectations: Vec<T>) -> Self {
        Self {
            expectations: Rc::new(RefCell::new((0, expectations))),
        }
    }

    pub fn done(&mut self) {
        assert_eq!(self.expectations.borrow().0, self.expectations.borrow().1.len());
    }
}

impl<T: Clone + Debug + PartialEq> Clone for GenericMock<T> {
    fn clone(&self) -> Self {
        Self {
            expectations: self.expectations.clone(),
        }
    }
}

impl<T: Clone + Debug + PartialEq> Iterator for GenericMock<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        (*self.expectations).borrow_mut().0 += 1;
        self.expectations.borrow().1.get(self.expectations.borrow().0 - 1).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec;

    #[test]
    fn generic_mock_test() {
        let expectations = vec![0x00, 0xff];
        let mut mock: GenericMock<u8> = GenericMock::new(expectations);

        assert_eq!(mock.next(), Some(0x00));
        assert_eq!(mock.next(), Some(0xff));
        assert_eq!(mock.next(), None);
    }
}
