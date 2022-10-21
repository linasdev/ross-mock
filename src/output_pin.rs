extern crate alloc;

use embedded_hal::digital::v2::OutputPin;

use crate::GenericMock;

#[derive(Debug)]
pub struct Infallible;

#[derive(Debug, Clone, PartialEq)]
pub enum Expectation {
    SetHigh,
    SetLow,
}

pub type Mock = GenericMock<Expectation>;

impl OutputPin for Mock {
    type Error = Infallible;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let mut s = self.clone();

        if let Some(Expectation::SetHigh) = s.next() {
            Ok(())
        } else {
            panic!("Did not expect call to set_high");
        }
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        let mut s = self.clone();

        if let Some(Expectation::SetLow) = s.next() {
            Ok(())
        } else {
            panic!("Did not expect call to set_low");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec;

    #[test]
    #[should_panic(expected = "Did not expect call to set_high")]
    fn mock_unexpected_call_to_set_high_test() {
        let mut mock = Mock::new(vec![]);

        mock.set_high().unwrap();
    }

    #[test]
    fn mock_set_high_test() {
        let mut mock = Mock::new(vec![Expectation::SetHigh]);

        mock.set_high().unwrap();
        mock.done();
    }

    #[test]
    #[should_panic(expected = "Did not expect call to set_low")]
    fn mock_unexpected_call_to_set_low_test() {
        let mut mock = Mock::new(vec![]);

        mock.set_low().unwrap();
    }

    #[test]
    fn mock_set_low_test() {
        let mut mock = Mock::new(vec![Expectation::SetLow]);

        mock.set_low().unwrap();
        mock.done();
    }

    #[test]
    fn mock_combination_test() {
        let mut mock = Mock::new(vec![
            Expectation::SetHigh,
            Expectation::SetLow,
            Expectation::SetHigh,
        ]);

        mock.set_high().unwrap();
        mock.set_low().unwrap();
        mock.set_high().unwrap();
        mock.done();
    }
}
