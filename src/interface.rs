extern crate alloc;

use ross_protocol::interface::{Interface, InterfaceError};
use ross_protocol::packet::Packet;

use crate::GenericMock;

#[derive(Debug, Clone, PartialEq)]
pub enum Expectation {
    SentPacket(Packet),
    ReceivedPacket(Packet),
}

pub type Mock = GenericMock<Expectation>;

impl Interface for Mock {
    fn try_get_packet(&mut self) -> Result<Packet, InterfaceError> {
        if let Some(Expectation::ReceivedPacket(packet)) = self.next() {
            Ok(packet)
        } else {
            panic!("Did not expect call to try_get_packet");
        }
    }

    fn try_send_packet(&mut self, packet: &Packet) -> Result<(), InterfaceError> {
        if let Some(Expectation::SentPacket(expected_packet)) = self.next() {
            assert_eq!(expected_packet.is_error, packet.is_error);
            assert_eq!(expected_packet.device_address, packet.device_address);
            assert_eq!(expected_packet.data, packet.data);

            Ok(())
        } else {
            panic!("Did not expect call to try_send_packet");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec;

    #[test]
    #[should_panic(expected = "Did not expect call to try_get_packet")]
    fn mock_unexpected_call_to_try_get_packet_test() {
        let mut mock = Mock::new(vec![]);

        mock.try_get_packet().unwrap();
    }
    
    #[test]
    fn mock_received_packet_test() {
        let mut mock = Mock::new(vec![
            Expectation::ReceivedPacket(Packet {
                is_error: true,
                device_address: 0x1111,
                data: vec![0x11, 0x11, 0x11],
            })
        ]);

        let packet = mock.try_get_packet().unwrap();

        assert_eq!(packet.is_error, true);
        assert_eq!(packet.device_address, 0x1111);
        assert_eq!(packet.data, vec![0x11, 0x11, 0x11]);

        mock.done();
    }

    #[test]
    #[should_panic(expected = "Did not expect call to try_send_packet")]
    fn mock_unexpected_call_to_try_send_packet_test() {
        let mut mock = Mock::new(vec![]);

        mock.try_send_packet(&Packet {
            is_error: true,
            device_address: 0x1111,
            data: vec![0x11, 0x11, 0x11],
        }).unwrap();
    }
    
    #[test]
    fn mock_sent_packet_test() {
        let mut mock = Mock::new(vec![
            Expectation::SentPacket(Packet {
                is_error: true,
                device_address: 0x1111,
                data: vec![0x11, 0x11, 0x11],
            })
        ]);

        mock.try_send_packet(&Packet {
            is_error: true,
            device_address: 0x1111,
            data: vec![0x11, 0x11, 0x11],
        }).unwrap();

        mock.done();
    }
    
    #[test]
    fn mock_combination_test() {
        let mut mock = Mock::new(vec![
            Expectation::SentPacket(Packet {
                is_error: true,
                device_address: 0x1111,
                data: vec![0x11, 0x11, 0x11],
            }),
            Expectation::ReceivedPacket(Packet {
                is_error: false,
                device_address: 0x2222,
                data: vec![0x22, 0x22, 0x22],
            }),
            Expectation::SentPacket(Packet {
                is_error: true,
                device_address: 0x3333,
                data: vec![0x33, 0x33, 0x33],
            }),
        ]);

        mock.try_send_packet(&Packet {
            is_error: true,
            device_address: 0x1111,
            data: vec![0x11, 0x11, 0x11],
        }).unwrap();


        let packet = mock.try_get_packet().unwrap();

        assert_eq!(packet.is_error, false);
        assert_eq!(packet.device_address, 0x2222);
        assert_eq!(packet.data, vec![0x22, 0x22, 0x22]);

        mock.try_send_packet(&Packet {
            is_error: true,
            device_address: 0x3333,
            data: vec![0x33, 0x33, 0x33],
        }).unwrap();

        mock.done();
    }
}
