#![no_std]

extern crate alloc;

use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt::Debug;
use embedded_hal::digital::v2::{InputPin, OutputPin};

use ross_protocol::interface::{Interface, InterfaceError};
use ross_protocol::packet::Packet;

#[derive(Debug)]
pub struct Infallible;

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
    expectations: Rc<RefCell<(usize, Vec<Expectation>)>>,
}

impl ExpectationTracker {
    pub fn new(expectations: Vec<Expectation>) -> Self {
        Self {
            expectations: Rc::new(RefCell::new((0, expectations))),
        }
    }

    pub fn done(&mut self) {
        assert_eq!(self.expectations.borrow().0, self.expectations.borrow().1.len());
    }
}

impl Clone for ExpectationTracker {
    fn clone(&self) -> Self {
        Self {
            expectations: self.expectations.clone(),
        }
    }
}

impl Iterator for ExpectationTracker {
    type Item = Expectation;

    fn next(&mut self) -> Option<Self::Item> {
        (*self.expectations).borrow_mut().0 += 1;
        self.expectations.borrow().1.get(self.expectations.borrow().0 - 1).cloned()
    }
}

impl Interface for ExpectationTracker {
    fn try_get_packet(&mut self) -> Result<Packet, InterfaceError> {
        let expectation_option = self.next();

        if let Some(Expectation::Interface(InterfaceExpectation::ReceivedPacket(packet))) = expectation_option {
            Ok(packet)
        } else if let Some(expectation) = expectation_option {
            panic!("Did not expect call to try_get_packet, expected: {:?}", expectation);
        } else {
            panic!("Did not expect call to try_get_packet, nothing was expected")
        }
    }

    fn try_send_packet(&mut self, packet: &Packet) -> Result<(), InterfaceError> {
        let expectation_option = self.next();

        if let Some(Expectation::Interface(InterfaceExpectation::SentPacket(expected_packet))) = expectation_option {
            assert_eq!(expected_packet.is_error, packet.is_error);
            assert_eq!(expected_packet.device_address, packet.device_address);
            assert_eq!(expected_packet.data, packet.data);

            Ok(())
        } else if let Some(expectation) = expectation_option {
            panic!("Did not expect call to try_send_packet, expected: {:?}", expectation);
        } else {
            panic!("Did not expect call to try_send_packet, nothing was expected");
        }
    }
}

impl InputPin for ExpectationTracker {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        let expectation_option = self.clone().next();

        if let Some(Expectation::InputPin(expectation)) = expectation_option {
            match expectation {
                InputPinExpectation::IsHigh => Ok(true),
                InputPinExpectation::IsLow => Ok(false),
            }
        } else if let Some(expectation) = expectation_option {
            panic!("Did not expect call to is_high, expected: {:?}", expectation);
        } else {
            panic!("Did not expect call to is_high, nothing was expected")
        }
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        let expectation_option = self.clone().next();

        if let Some(Expectation::InputPin(expectation)) = expectation_option {
            match expectation {
                InputPinExpectation::IsHigh => Ok(false),
                InputPinExpectation::IsLow => Ok(true),
            }
        } else if let Some(expectation) = expectation_option {
            panic!("Did not expect call to is_low, expected: {:?}", expectation);
        } else {
            panic!("Did not expect call to is_low, nothing was expected")
        }
    }
}

impl OutputPin for ExpectationTracker {
    type Error = Infallible;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let expectation_option = self.next();

        if let Some(Expectation::OutputPin(OutputPinExpectation::SetHigh)) = expectation_option {
            Ok(())
        } else if let Some(expectation) = expectation_option {
            panic!("Did not expect call to set_high, expected: {:?}", expectation);
        } else {
            panic!("Did not expect call to set_high, nothing was expected")
        }
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        let expectation_option = self.next();

        if let Some(Expectation::OutputPin(OutputPinExpectation::SetLow)) = expectation_option {
            Ok(())
        } else if let Some(expectation) = expectation_option {
            panic!("Did not expect call to set_low, expected: {:?}", expectation);
        } else {
            panic!("Did not expect call to set_low, nothing was expected")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec;

    #[test]
    #[should_panic(expected = "Did not expect call to try_get_packet, nothing was expected")]
    fn expectation_tracker_unexpected_call_to_try_get_packet_test() {
        let expectations = vec![];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        expectation_tracker.try_get_packet().unwrap();
    }
    
    #[test]
    fn expectation_tracker_received_packet_test() {
        let expectations = vec![
            Expectation::Interface(InterfaceExpectation::ReceivedPacket(Packet {
                is_error: true,
                device_address: 0x1111,
                data: vec![0x11, 0x11, 0x11],
            }))
        ];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        let packet = expectation_tracker.try_get_packet().unwrap();

        assert_eq!(packet.is_error, true);
        assert_eq!(packet.device_address, 0x1111);
        assert_eq!(packet.data, vec![0x11, 0x11, 0x11]);

        expectation_tracker.done();
    }

    #[test]
    #[should_panic(expected = "Did not expect call to try_send_packet, nothing was expected")]
    fn expectation_tracker_unexpected_call_to_try_send_packet_test() {
        let expectations = vec![];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        expectation_tracker.try_send_packet(&Packet {
            is_error: true,
            device_address: 0x1111,
            data: vec![0x11, 0x11, 0x11],
        }).unwrap();
    }
    
    #[test]
    fn expectation_tracker_sent_packet_test() {
        let expectations = vec![
            Expectation::Interface(InterfaceExpectation::SentPacket(Packet {
                is_error: true,
                device_address: 0x1111,
                data: vec![0x11, 0x11, 0x11],
            }))
        ];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        expectation_tracker.try_send_packet(&Packet {
            is_error: true,
            device_address: 0x1111,
            data: vec![0x11, 0x11, 0x11],
        }).unwrap();

        expectation_tracker.done();
    }
    
    #[test]
    fn expectation_tracker_interface_expectation_combination_test() {
        let expectations = vec![
            Expectation::Interface(InterfaceExpectation::SentPacket(Packet {
                is_error: true,
                device_address: 0x1111,
                data: vec![0x11, 0x11, 0x11],
            })),
            Expectation::Interface(InterfaceExpectation::ReceivedPacket(Packet {
                is_error: false,
                device_address: 0x2222,
                data: vec![0x22, 0x22, 0x22],
            })),
            Expectation::Interface(InterfaceExpectation::SentPacket(Packet {
                is_error: true,
                device_address: 0x3333,
                data: vec![0x33, 0x33, 0x33],
            })),
        ];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        expectation_tracker.try_send_packet(&Packet {
            is_error: true,
            device_address: 0x1111,
            data: vec![0x11, 0x11, 0x11],
        }).unwrap();

        let packet = expectation_tracker.try_get_packet().unwrap();

        assert_eq!(packet.is_error, false);
        assert_eq!(packet.device_address, 0x2222);
        assert_eq!(packet.data, vec![0x22, 0x22, 0x22]);

        expectation_tracker.try_send_packet(&Packet {
            is_error: true,
            device_address: 0x3333,
            data: vec![0x33, 0x33, 0x33],
        }).unwrap();

        expectation_tracker.done();
    }

    #[test]
    #[should_panic(expected = "Did not expect call to is_high, nothing was expected")]
    fn expectation_tracker_unexpected_call_to_is_high_test() {
        let expectations = vec![];
        let expectation_tracker = ExpectationTracker::new(expectations);

        expectation_tracker.is_high().unwrap();
    }

    #[test]
    fn expectation_tracker_call_to_is_high_is_high_test() {
        let expectations = vec![
            Expectation::InputPin(InputPinExpectation::IsHigh)
        ];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        let result = expectation_tracker.is_high().unwrap();
        assert_eq!(result, true);

        expectation_tracker.done();
    }

    #[test]
    fn expectation_tracker_call_to_is_high_is_low_test() {
        let expectations = vec![
            Expectation::InputPin(InputPinExpectation::IsLow)
        ];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        let result = expectation_tracker.is_high().unwrap();
        assert_eq!(result, false);

        expectation_tracker.done();
    }

    #[test]
    #[should_panic(expected = "Did not expect call to is_low, nothing was expected")]
    fn expectation_tracker_unexpected_call_to_is_low_test() {
        let expectations = vec![];
        let expectation_tracker = ExpectationTracker::new(expectations);

        expectation_tracker.is_low().unwrap();
    }

    #[test]
    fn expectation_tracker_call_to_is_low_is_low_test() {
        let expectations = vec![
            Expectation::InputPin(InputPinExpectation::IsLow)
        ];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        let result = expectation_tracker.is_low().unwrap();
        assert_eq!(result, true);

        expectation_tracker.done();
    }

    #[test]
    fn expectation_tracker_call_to_is_low_is_high_test() {
        let expectations = vec![
            Expectation::InputPin(InputPinExpectation::IsHigh)
        ];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        let result = expectation_tracker.is_low().unwrap();
        assert_eq!(result, false);

        expectation_tracker.done();
    }

    #[test]
    fn expectation_tracker_input_pin_expectation_combination_test() {
        let expectations = vec![
            Expectation::InputPin(InputPinExpectation::IsHigh),
            Expectation::InputPin(InputPinExpectation::IsLow),
            Expectation::InputPin(InputPinExpectation::IsHigh),
        ];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        let result = expectation_tracker.is_high().unwrap();
        assert_eq!(result, true);

        let result = expectation_tracker.is_high().unwrap();
        assert_eq!(result, false);

        let result = expectation_tracker.is_low().unwrap();
        assert_eq!(result, false);

        expectation_tracker.done();
    }

    #[test]
    #[should_panic(expected = "Did not expect call to set_high, nothing was expected")]
    fn expectation_tracker_unexpected_call_to_set_high_test() {
        let expectations = vec![];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        expectation_tracker.set_high().unwrap();
    }

    #[test]
    fn expectation_tracker_set_high_test() {
        let expectations = vec![
            Expectation::OutputPin(OutputPinExpectation::SetHigh),
        ];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        expectation_tracker.set_high().unwrap();
        expectation_tracker.done();
    }

    #[test]
    #[should_panic(expected = "Did not expect call to set_low, nothing was expected")]
    fn expectation_tracker_unexpected_call_to_set_low_test() {
        let expectations = vec![];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        expectation_tracker.set_low().unwrap();
    }

    #[test]
    fn expectation_tracker_set_low_test() {
        let expectations = vec![
            Expectation::OutputPin(OutputPinExpectation::SetLow),
        ];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        expectation_tracker.set_low().unwrap();
        expectation_tracker.done();
    }

    #[test]
    fn expectation_tracker_output_pin_expectation_combination_test() {
        let expectations = vec![
            Expectation::OutputPin(OutputPinExpectation::SetHigh),
            Expectation::OutputPin(OutputPinExpectation::SetLow),
            Expectation::OutputPin(OutputPinExpectation::SetHigh),
        ];
        let mut expectation_tracker = ExpectationTracker::new(expectations);

        expectation_tracker.set_high().unwrap();
        expectation_tracker.set_low().unwrap();
        expectation_tracker.set_high().unwrap();
        expectation_tracker.done();
    }
}
