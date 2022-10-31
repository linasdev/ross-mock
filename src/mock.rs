use alloc::rc::Rc;
use core::cell::RefCell;
use core::convert::Infallible;
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};

use ross_protocol::interface::{Interface, InterfaceError};
use ross_protocol::packet::Packet;

use crate::{
    Expectation, ExpectationTracker, InputPinExpectation, InterfaceExpectation,
    OutputPinExpectation,
};

#[derive(Debug)]
pub struct Mock {
    expectation_tracker: Rc<RefCell<ExpectationTracker>>,
    index: usize,
}

impl Mock {
    pub(crate) fn new(expectation_tracker: Rc<RefCell<ExpectationTracker>>, index: usize) -> Self {
        Self {
            expectation_tracker,
            index,
        }
    }

    pub(crate) fn get_index(&self) -> usize {
        self.index
    }
}

impl Interface for Mock {
    fn try_get_packet(&mut self) -> Result<Packet, InterfaceError> {
        let expectation_option = self.expectation_tracker.borrow_mut().next();

        if let Some((index, expectation)) = expectation_option {
            if self.index == index {
                if let Expectation::Interface(InterfaceExpectation::ReceivedPacket(packet)) =
                    expectation
                {
                    Ok(packet)
                } else {
                    panic!(
                        "Did not expect call to try_get_packet, expected: {:?}",
                        expectation
                    );
                }
            } else {
                panic!(
                    "Mock with index {} cannot verify expectation with index {}",
                    self.index, index
                );
            }
        } else {
            panic!("Did not expect call to try_get_packet, nothing was expected")
        }
    }

    fn try_send_packet(&mut self, packet: &Packet) -> Result<(), InterfaceError> {
        let expectation_option = self.expectation_tracker.borrow_mut().next();

        if let Some((index, expectation)) = expectation_option {
            if self.index == index {
                if let Expectation::Interface(InterfaceExpectation::SentPacket(expected_packet)) =
                    expectation
                {
                    assert_eq!(expected_packet.is_error, packet.is_error);
                    assert_eq!(expected_packet.device_address, packet.device_address);
                    assert_eq!(expected_packet.data, packet.data);
                    Ok(())
                } else {
                    panic!(
                        "Did not expect call to try_send_packet, expected: {:?}",
                        expectation
                    );
                }
            } else {
                panic!(
                    "Mock with index {} cannot verify expectation with index {}",
                    self.index, index
                );
            }
        } else {
            panic!("Did not expect call to try_send_packet, nothing was expected")
        }
    }
}

impl InputPin for Mock {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        let expectation_option = self.expectation_tracker.borrow_mut().next();

        if let Some((index, expectation)) = expectation_option {
            if self.index == index {
                if let Expectation::InputPin(input_pin_expectation) = expectation {
                    match input_pin_expectation {
                        InputPinExpectation::IsHigh => Ok(true),
                        InputPinExpectation::IsLow => Ok(false),
                    }
                } else {
                    panic!(
                        "Did not expect call to is_high, expected: {:?}",
                        expectation
                    );
                }
            } else {
                panic!(
                    "Mock with index {} cannot verify expectation with index {}",
                    self.index, index
                );
            }
        } else {
            panic!("Did not expect call to is_high, nothing was expected")
        }
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        let expectation_option = self.expectation_tracker.borrow_mut().next();

        if let Some((index, expectation)) = expectation_option {
            if self.index == index {
                if let Expectation::InputPin(input_pin_expectation) = expectation {
                    match input_pin_expectation {
                        InputPinExpectation::IsHigh => Ok(false),
                        InputPinExpectation::IsLow => Ok(true),
                    }
                } else {
                    panic!("Did not expect call to is_low, expected: {:?}", expectation);
                }
            } else {
                panic!(
                    "Mock with index {} cannot verify expectation with index {}",
                    self.index, index
                );
            }
        } else {
            panic!("Did not expect call to is_low, nothing was expected")
        }
    }
}

impl OutputPin for Mock {
    type Error = Infallible;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let expectation_option = self.expectation_tracker.borrow_mut().next();

        if let Some((index, expectation)) = expectation_option {
            if self.index == index {
                if let Expectation::OutputPin(OutputPinExpectation::SetHigh) = expectation {
                    Ok(())
                } else {
                    panic!(
                        "Did not expect call to set_high, expected: {:?}",
                        expectation
                    );
                }
            } else {
                panic!(
                    "Mock with index {} cannot verify expectation with index {}",
                    self.index, index
                );
            }
        } else {
            panic!("Did not expect call to set_high, nothing was expected")
        }
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        let expectation_option = self.expectation_tracker.borrow_mut().next();

        if let Some((index, expectation)) = expectation_option {
            if self.index == index {
                if let Expectation::OutputPin(OutputPinExpectation::SetLow) = expectation {
                    Ok(())
                } else {
                    panic!(
                        "Did not expect call to set_low, expected: {:?}",
                        expectation
                    );
                }
            } else {
                panic!(
                    "Mock with index {} cannot verify expectation with index {}",
                    self.index, index
                );
            }
        } else {
            panic!("Did not expect call to set_low, nothing was expected")
        }
    }
}

impl StatefulOutputPin for Mock {
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(true)
    }
}
