extern crate alloc;

use embedded_hal::digital::v2::InputPin;

use crate::GenericMock;

#[derive(Debug)]
pub struct Infallible;

#[derive(Debug, Clone, PartialEq)]
pub enum Expectation {
    IsHigh,
    IsLow,
}

pub type Mock = GenericMock<Expectation>;

impl InputPin for Mock {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        let mut s = self.clone();

        if let Some(expectation) = s.next() {
            match expectation {
                Expectation::IsHigh => Ok(true),
                Expectation::IsLow => Ok(false),
            }
        } else {
            panic!("Did not expect call to is_high");
        }
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        let mut s = self.clone();

        if let Some(expectation) = s.next() {
            match expectation {
                Expectation::IsHigh => Ok(false),
                Expectation::IsLow => Ok(true),
            }
        } else {
            panic!("Did not expect call to is_low");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec;

    #[test]
    #[should_panic(expected = "Did not expect call to is_high")]
    fn mock_unexpected_call_to_is_high_test() {
        let mock = Mock::new(vec![]);

        mock.is_high().unwrap();
    }

    #[test]
    fn mock_call_to_is_high_is_high_test() {
        let mut mock = Mock::new(vec![Expectation::IsHigh]);

        let result = mock.is_high().unwrap();
        assert_eq!(result, true);

        mock.done();
    }

    #[test]
    fn mock_call_to_is_high_is_low_test() {
        let mut mock = Mock::new(vec![Expectation::IsLow]);

        let result = mock.is_high().unwrap();
        assert_eq!(result, false);

        mock.done();
    }

    #[test]
    #[should_panic(expected = "Did not expect call to is_low")]
    fn mock_unexpected_call_to_is_low_test() {
        let mock = Mock::new(vec![]);

        mock.is_low().unwrap();
    }

    #[test]
    fn mock_call_to_is_low_is_low_test() {
        let mut mock = Mock::new(vec![Expectation::IsLow]);

        let result = mock.is_low().unwrap();
        assert_eq!(result, true);

        mock.done();
    }

    #[test]
    fn mock_call_to_is_low_is_high_test() {
        let mut mock = Mock::new(vec![Expectation::IsHigh]);

        let result = mock.is_low().unwrap();
        assert_eq!(result, false);

        mock.done();
    }

    #[test]
    fn mock_combination_test() {
        let mut mock = Mock::new(vec![
            Expectation::IsHigh,
            Expectation::IsLow,
            Expectation::IsHigh,
        ]);

        let result = mock.is_high().unwrap();
        assert_eq!(result, true);

        let result = mock.is_high().unwrap();
        assert_eq!(result, false);

        let result = mock.is_low().unwrap();
        assert_eq!(result, false);

        mock.done();
    }
}
