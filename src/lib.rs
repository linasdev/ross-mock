#![no_std]

extern crate alloc;

use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt::Debug;

use ross_protocol::packet::Packet;

mod mock;
pub use mock::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Expectation {
    Interface(InterfaceExpectation),
    InputPin(InputPinExpectation),
    OutputPin(OutputPinExpectation),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceExpectation {
    SentPacket(Packet),
    ReceivedPacket(Packet),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputPinExpectation {
    IsHigh,
    IsLow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputPinExpectation {
    SetHigh,
    SetLow,
}

#[derive(Debug)]
pub struct ExpectationTracker {
    expectations: Rc<RefCell<(usize, Vec<(usize, Expectation)>)>>,
    mock_index: usize,
}

impl ExpectationTracker {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            expectations: Rc::new(RefCell::new((0, vec![]))),
            mock_index: 0,
        }))
    }

    pub fn expect(tracker: Rc<RefCell<ExpectationTracker>>, mock: &Mock, expectation: Expectation) {
        tracker
            .borrow_mut()
            .expectations
            .borrow_mut()
            .1
            .push((mock.get_index(), expectation));
    }

    pub fn mock(tracker: Rc<RefCell<ExpectationTracker>>) -> Mock {
        Mock::new(tracker.clone(), tracker.borrow().mock_index)
    }

    pub fn done(&mut self) {
        assert_eq!(
            self.expectations.borrow().0,
            self.expectations.borrow().1.len()
        );
    }
}

impl Clone for ExpectationTracker {
    fn clone(&self) -> Self {
        Self {
            expectations: self.expectations.clone(),
            mock_index: self.mock_index,
        }
    }
}

impl Iterator for ExpectationTracker {
    type Item = (usize, Expectation);

    fn next(&mut self) -> Option<Self::Item> {
        (*self.expectations).borrow_mut().0 += 1;
        self.expectations
            .borrow()
            .1
            .get(self.expectations.borrow().0 - 1)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec;

    use embedded_hal::digital::v2::{InputPin, OutputPin};
    use ross_protocol::interface::Interface;

    #[test]
    #[should_panic(expected = "Did not expect call to try_get_packet, nothing was expected")]
    fn unexpected_call_to_try_get_packet_test() {
        let tracker = ExpectationTracker::new();
        let mut mock = ExpectationTracker::mock(tracker.clone());

        mock.try_get_packet().unwrap();
    }

    #[test]
    fn received_packet_test() {
        let tracker = ExpectationTracker::new();
        let mut mock = ExpectationTracker::mock(tracker.clone());

        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::Interface(InterfaceExpectation::ReceivedPacket(Packet {
                is_error: true,
                device_address: 0x1111,
                data: vec![0x11, 0x11, 0x11],
            })),
        );

        let packet = mock.try_get_packet().unwrap();

        assert_eq!(packet.is_error, true);
        assert_eq!(packet.device_address, 0x1111);
        assert_eq!(packet.data, vec![0x11, 0x11, 0x11]);

        tracker.borrow_mut().done();
    }

    #[test]
    #[should_panic(expected = "Did not expect call to try_send_packet, nothing was expected")]
    fn unexpected_call_to_try_send_packet_test() {
        let tracker = ExpectationTracker::new();
        let mut mock = ExpectationTracker::mock(tracker.clone());

        mock.try_send_packet(&Packet {
            is_error: true,
            device_address: 0x1111,
            data: vec![0x11, 0x11, 0x11],
        })
        .unwrap();
    }

    #[test]
    fn sent_packet_test() {
        let tracker = ExpectationTracker::new();
        let mut mock = ExpectationTracker::mock(tracker.clone());

        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::Interface(InterfaceExpectation::SentPacket(Packet {
                is_error: true,
                device_address: 0x1111,
                data: vec![0x11, 0x11, 0x11],
            })),
        );

        mock.try_send_packet(&Packet {
            is_error: true,
            device_address: 0x1111,
            data: vec![0x11, 0x11, 0x11],
        })
        .unwrap();

        tracker.borrow_mut().done();
    }

    #[test]
    fn interface_expectation_combination_test() {
        let tracker = ExpectationTracker::new();
        let mut mock = ExpectationTracker::mock(tracker.clone());

        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::Interface(InterfaceExpectation::SentPacket(Packet {
                is_error: true,
                device_address: 0x1111,
                data: vec![0x11, 0x11, 0x11],
            })),
        );
        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::Interface(InterfaceExpectation::ReceivedPacket(Packet {
                is_error: false,
                device_address: 0x2222,
                data: vec![0x22, 0x22, 0x22],
            })),
        );
        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::Interface(InterfaceExpectation::SentPacket(Packet {
                is_error: true,
                device_address: 0x3333,
                data: vec![0x33, 0x33, 0x33],
            })),
        );

        mock.try_send_packet(&Packet {
            is_error: true,
            device_address: 0x1111,
            data: vec![0x11, 0x11, 0x11],
        })
        .unwrap();

        let packet = mock.try_get_packet().unwrap();

        assert_eq!(packet.is_error, false);
        assert_eq!(packet.device_address, 0x2222);
        assert_eq!(packet.data, vec![0x22, 0x22, 0x22]);

        mock.try_send_packet(&Packet {
            is_error: true,
            device_address: 0x3333,
            data: vec![0x33, 0x33, 0x33],
        })
        .unwrap();

        tracker.borrow_mut().done();
    }

    #[test]
    #[should_panic(expected = "Did not expect call to is_high, nothing was expected")]
    fn unexpected_call_to_is_high_test() {
        let tracker = ExpectationTracker::new();
        let mock = ExpectationTracker::mock(tracker.clone());

        mock.is_high().unwrap();
    }

    #[test]
    fn call_to_is_high_is_high_test() {
        let tracker = ExpectationTracker::new();
        let mock = ExpectationTracker::mock(tracker.clone());

        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::InputPin(InputPinExpectation::IsHigh),
        );

        let result = mock.is_high().unwrap();
        assert_eq!(result, true);

        tracker.borrow_mut().done();
    }

    #[test]
    fn call_to_is_high_is_low_test() {
        let tracker = ExpectationTracker::new();
        let mock = ExpectationTracker::mock(tracker.clone());

        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::InputPin(InputPinExpectation::IsLow),
        );

        let result = mock.is_high().unwrap();
        assert_eq!(result, false);

        tracker.borrow_mut().done();
    }

    #[test]
    #[should_panic(expected = "Did not expect call to is_low, nothing was expected")]
    fn unexpected_call_to_is_low_test() {
        let tracker = ExpectationTracker::new();
        let mock = ExpectationTracker::mock(tracker.clone());

        mock.is_low().unwrap();
    }

    #[test]
    fn call_to_is_low_is_low_test() {
        let tracker = ExpectationTracker::new();
        let mock = ExpectationTracker::mock(tracker.clone());

        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::InputPin(InputPinExpectation::IsLow),
        );

        let result = mock.is_low().unwrap();
        assert_eq!(result, true);

        tracker.borrow_mut().done();
    }

    #[test]
    fn call_to_is_low_is_high_test() {
        let tracker = ExpectationTracker::new();
        let mock = ExpectationTracker::mock(tracker.clone());

        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::InputPin(InputPinExpectation::IsHigh),
        );

        let result = mock.is_low().unwrap();
        assert_eq!(result, false);

        tracker.borrow_mut().done();
    }

    #[test]
    fn input_pin_expectation_combination_test() {
        let tracker = ExpectationTracker::new();
        let mock = ExpectationTracker::mock(tracker.clone());

        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::InputPin(InputPinExpectation::IsHigh),
        );
        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::InputPin(InputPinExpectation::IsLow),
        );
        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::InputPin(InputPinExpectation::IsHigh),
        );

        let result = mock.is_high().unwrap();
        assert_eq!(result, true);

        let result = mock.is_high().unwrap();
        assert_eq!(result, false);

        let result = mock.is_low().unwrap();
        assert_eq!(result, false);

        tracker.borrow_mut().done();
    }

    #[test]
    #[should_panic(expected = "Did not expect call to set_high, nothing was expected")]
    fn unexpected_call_to_set_high_test() {
        let tracker = ExpectationTracker::new();
        let mut mock = ExpectationTracker::mock(tracker.clone());

        mock.set_high().unwrap();
    }

    #[test]
    fn set_high_test() {
        let tracker = ExpectationTracker::new();
        let mut mock = ExpectationTracker::mock(tracker.clone());

        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::OutputPin(OutputPinExpectation::SetHigh),
        );

        mock.set_high().unwrap();
        tracker.borrow_mut().done();
    }

    #[test]
    #[should_panic(expected = "Did not expect call to set_low, nothing was expected")]
    fn unexpected_call_to_set_low_test() {
        let tracker = ExpectationTracker::new();
        let mut mock = ExpectationTracker::mock(tracker.clone());

        mock.set_low().unwrap();
    }

    #[test]
    fn set_low_test() {
        let tracker = ExpectationTracker::new();
        let mut mock = ExpectationTracker::mock(tracker.clone());

        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::OutputPin(OutputPinExpectation::SetLow),
        );

        mock.set_low().unwrap();
        tracker.borrow_mut().done();
    }

    #[test]
    fn output_pin_expectation_combination_test() {
        let tracker = ExpectationTracker::new();
        let mut mock = ExpectationTracker::mock(tracker.clone());

        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::OutputPin(OutputPinExpectation::SetHigh),
        );
        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::OutputPin(OutputPinExpectation::SetLow),
        );
        ExpectationTracker::expect(
            tracker.clone(),
            &mock,
            Expectation::OutputPin(OutputPinExpectation::SetHigh),
        );

        mock.set_high().unwrap();
        mock.set_low().unwrap();
        mock.set_high().unwrap();

        tracker.borrow_mut().done();
    }

    #[test]
    fn input_pin_output_pin_expectation_combination_test() {
        let tracker = ExpectationTracker::new();
        let input_pin_mock = ExpectationTracker::mock(tracker.clone());
        let mut output_pin_mock = ExpectationTracker::mock(tracker.clone());

        ExpectationTracker::expect(
            tracker.clone(),
            &input_pin_mock,
            Expectation::InputPin(InputPinExpectation::IsHigh),
        );
        ExpectationTracker::expect(
            tracker.clone(),
            &output_pin_mock,
            Expectation::OutputPin(OutputPinExpectation::SetLow),
        );
        ExpectationTracker::expect(
            tracker.clone(),
            &input_pin_mock,
            Expectation::InputPin(InputPinExpectation::IsLow),
        );

        let result = input_pin_mock.is_high().unwrap();
        assert_eq!(result, true);

        output_pin_mock.set_low().unwrap();

        let result = input_pin_mock.is_low().unwrap();
        assert_eq!(result, true);

        tracker.borrow_mut().done();
    }
}
