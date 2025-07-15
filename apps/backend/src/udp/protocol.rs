use anyhow::Result;
use std::io::{Cursor, Read, Write};

// Taille maximale d'un paquet UDP (MTU typique - headers IP/UDP)
pub const MAX_PACKET_SIZE: usize = 1472;

// Types de paquets
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PacketType {
    Connect = 0x01,
    Disconnect = 0x02,
    Ping = 0x03,
    Pong = 0x04,
    Ack = 0x05,
    Command = 0x10,
    FrameData = 0x20,
    FrameDataCompressed = 0x21,
    SpectrumData = 0x30,
}

impl PacketType {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Connect),
            0x02 => Some(Self::Disconnect),
            0x03 => Some(Self::Ping),
            0x04 => Some(Self::Pong),
            0x05 => Some(Self::Ack),
            0x10 => Some(Self::Command),
            0x20 => Some(Self::FrameData),
            0x21 => Some(Self::FrameDataCompressed),
            0x30 => Some(Self::SpectrumData),
            _ => None,
        }
    }
}

// Flags pour les options du paquet
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PacketFlags: u8 {
        const NONE = 0x00;
        const COMPRESSED = 0x01;
        const FRAGMENTED = 0x02;
        const LAST_FRAGMENT = 0x04;
        const REQUIRES_ACK = 0x08;
    }
}

// Structure d'un paquet UDP
#[derive(Debug, Clone)]
pub struct UdpPacket {
    pub packet_type: PacketType,
    pub flags: PacketFlags,
    pub sequence: u32,
    pub fragment_id: u16,
    pub fragment_count: u16,
    pub payload: Vec<u8>,
}

impl UdpPacket {
    pub fn new(packet_type: PacketType, sequence: u32, payload: Vec<u8>) -> Self {
        Self {
            packet_type,
            flags: PacketFlags::NONE,
            sequence,
            fragment_id: 0,
            fragment_count: 1,
            payload,
        }
    }

    pub fn new_connect(compression_enabled: bool) -> Self {
        let mut flags = PacketFlags::NONE;
        if compression_enabled {
            flags |= PacketFlags::COMPRESSED;
        }

        Self {
            packet_type: PacketType::Connect,
            flags,
            sequence: 0,
            fragment_id: 0,
            fragment_count: 1,
            payload: vec![],
        }
    }

    pub fn new_ack(sequence: u32) -> Self {
        Self {
            packet_type: PacketType::Ack,
            flags: PacketFlags::NONE,
            sequence,
            fragment_id: 0,
            fragment_count: 1,
            payload: vec![],
        }
    }

    pub fn new_pong(sequence: u32) -> Self {
        Self {
            packet_type: PacketType::Pong,
            flags: PacketFlags::NONE,
            sequence,
            fragment_id: 0,
            fragment_count: 1,
            payload: vec![],
        }
    }

    // Sérialisation en bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::with_capacity(16 + self.payload.len());
        let mut cursor = Cursor::new(&mut buffer);

        // Header (12 bytes)
        cursor.write_all(&[self.packet_type as u8])?;
        cursor.write_all(&[self.flags.bits()])?;
        cursor.write_all(&self.sequence.to_le_bytes())?;
        cursor.write_all(&self.fragment_id.to_le_bytes())?;
        cursor.write_all(&self.fragment_count.to_le_bytes())?;
        cursor.write_all(&(self.payload.len() as u16).to_le_bytes())?;

        // Payload
        cursor.write_all(&self.payload)?;

        Ok(buffer)
    }

    // Désérialisation depuis bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 12 {
            anyhow::bail!("Packet too small");
        }

        let mut cursor = Cursor::new(data);

        // Lire le header
        let mut type_byte = [0u8; 1];
        cursor.read_exact(&mut type_byte)?;
        let packet_type = PacketType::from_u8(type_byte[0])
            .ok_or_else(|| anyhow::anyhow!("Invalid packet type"))?;

        let mut flags_byte = [0u8; 1];
        cursor.read_exact(&mut flags_byte)?;
        let flags = PacketFlags::from_bits_truncate(flags_byte[0]);

        let mut sequence_bytes = [0u8; 4];
        cursor.read_exact(&mut sequence_bytes)?;
        let sequence = u32::from_le_bytes(sequence_bytes);

        let mut fragment_id_bytes = [0u8; 2];
        cursor.read_exact(&mut fragment_id_bytes)?;
        let fragment_id = u16::from_le_bytes(fragment_id_bytes);

        let mut fragment_count_bytes = [0u8; 2];
        cursor.read_exact(&mut fragment_count_bytes)?;
        let fragment_count = u16::from_le_bytes(fragment_count_bytes);

        let mut payload_len_bytes = [0u8; 2];
        cursor.read_exact(&mut payload_len_bytes)?;
        let payload_len = u16::from_le_bytes(payload_len_bytes) as usize;

        // Lire le payload
        let mut payload = vec![0u8; payload_len];
        cursor.read_exact(&mut payload)?;

        Ok(Self {
            packet_type,
            flags,
            sequence,
            fragment_id,
            fragment_count,
            payload,
        })
    }
}

// Commandes possibles
#[derive(Debug, Clone)]
pub enum UdpCommand {
    SetEffect(usize),
    SetColorMode(String),
    SetCustomColor(f32, f32, f32),
    SetParameter(String, String),
}

impl UdpCommand {
    pub fn to_payload(&self) -> Vec<u8> {
        match self {
            Self::SetEffect(id) => {
                let mut data = vec![0x01]; // Command ID
                data.extend_from_slice(&(*id as u32).to_le_bytes());
                data
            }
            Self::SetColorMode(mode) => {
                let mut data = vec![0x02]; // Command ID
                data.extend_from_slice(mode.as_bytes());
                data
            }
            Self::SetCustomColor(r, g, b) => {
                let mut data = vec![0x03]; // Command ID
                data.extend_from_slice(&r.to_le_bytes());
                data.extend_from_slice(&g.to_le_bytes());
                data.extend_from_slice(&b.to_le_bytes());
                data
            }
            Self::SetParameter(name, value) => {
                let mut data = vec![0x04]; // Command ID
                data.extend_from_slice(&(name.len() as u16).to_le_bytes());
                data.extend_from_slice(name.as_bytes());
                data.extend_from_slice(&(value.len() as u16).to_le_bytes());
                data.extend_from_slice(value.as_bytes());
                data
            }
        }
    }

    pub fn from_payload(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        let mut cursor = Cursor::new(data);
        let mut cmd_id = [0u8; 1];
        cursor.read_exact(&mut cmd_id).ok()?;

        match cmd_id[0] {
            0x01 => {
                let mut id_bytes = [0u8; 4];
                cursor.read_exact(&mut id_bytes).ok()?;
                Some(Self::SetEffect(u32::from_le_bytes(id_bytes) as usize))
            }
            0x02 => {
                let mode = String::from_utf8(data[1..].to_vec()).ok()?;
                Some(Self::SetColorMode(mode))
            }
            0x03 => {
                let mut r_bytes = [0u8; 4];
                let mut g_bytes = [0u8; 4];
                let mut b_bytes = [0u8; 4];
                cursor.read_exact(&mut r_bytes).ok()?;
                cursor.read_exact(&mut g_bytes).ok()?;
                cursor.read_exact(&mut b_bytes).ok()?;
                Some(Self::SetCustomColor(
                    f32::from_le_bytes(r_bytes),
                    f32::from_le_bytes(g_bytes),
                    f32::from_le_bytes(b_bytes),
                ))
            }
            0x04 => {
                let mut name_len_bytes = [0u8; 2];
                cursor.read_exact(&mut name_len_bytes).ok()?;
                let name_len = u16::from_le_bytes(name_len_bytes) as usize;

                let mut name_bytes = vec![0u8; name_len];
                cursor.read_exact(&mut name_bytes).ok()?;
                let name = String::from_utf8(name_bytes).ok()?;

                let mut value_len_bytes = [0u8; 2];
                cursor.read_exact(&mut value_len_bytes).ok()?;
                let value_len = u16::from_le_bytes(value_len_bytes) as usize;

                let mut value_bytes = vec![0u8; value_len];
                cursor.read_exact(&mut value_bytes).ok()?;
                let value = String::from_utf8(value_bytes).ok()?;

                Some(Self::SetParameter(name, value))
            }
            _ => None,
        }
    }
}

// Structure pour les données de frame
#[derive(Debug, Clone)]
pub struct FrameData {
    pub width: u16,
    pub height: u16,
    pub format: FrameFormat,
    pub data: Vec<u8>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum FrameFormat {
    RGB = 0x01,
    RGBA = 0x02,
    BGR = 0x03,
    BGRA = 0x04,
}

impl FrameData {
    pub fn to_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(5 + self.data.len());
        payload.extend_from_slice(&self.width.to_le_bytes());
        payload.extend_from_slice(&self.height.to_le_bytes());
        payload.push(self.format as u8);
        payload.extend_from_slice(&self.data);
        payload
    }

    pub fn from_payload(data: &[u8]) -> Option<Self> {
        if data.len() < 5 {
            return None;
        }

        let width = u16::from_le_bytes([data[0], data[1]]);
        let height = u16::from_le_bytes([data[2], data[3]]);
        let format = match data[4] {
            0x01 => FrameFormat::RGB,
            0x02 => FrameFormat::RGBA,
            0x03 => FrameFormat::BGR,
            0x04 => FrameFormat::BGRA,
            _ => return None,
        };

        Some(Self {
            width,
            height,
            format,
            data: data[5..].to_vec(),
        })
    }
}

// Structure pour les données de spectre
#[derive(Debug, Clone)]
pub struct SpectrumData {
    pub bands: Vec<f32>,
}

impl SpectrumData {
    pub fn to_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(2 + self.bands.len() * 4);
        payload.extend_from_slice(&(self.bands.len() as u16).to_le_bytes());

        for &value in &self.bands {
            payload.extend_from_slice(&value.to_le_bytes());
        }

        payload
    }

    pub fn from_payload(data: &[u8]) -> Option<Self> {
        if data.len() < 2 {
            return None;
        }

        let band_count = u16::from_le_bytes([data[0], data[1]]) as usize;
        let expected_size = 2 + band_count * 4;

        if data.len() != expected_size {
            return None;
        }

        let mut bands = Vec::with_capacity(band_count);
        for i in 0..band_count {
            let offset = 2 + i * 4;
            let value = f32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            bands.push(value);
        }

        Some(Self { bands })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_serialization() {
        let packet = UdpPacket::new(PacketType::FrameData, 42, vec![1, 2, 3, 4, 5]);

        let bytes = packet.to_bytes().unwrap();
        let decoded = UdpPacket::from_bytes(&bytes).unwrap();

        assert_eq!(packet.packet_type, decoded.packet_type);
        assert_eq!(packet.sequence, decoded.sequence);
        assert_eq!(packet.payload, decoded.payload);
    }

    #[test]
    fn test_command_serialization() {
        let cmd = UdpCommand::SetEffect(5);
        let payload = cmd.to_payload();
        let decoded = UdpCommand::from_payload(&payload).unwrap();

        match decoded {
            UdpCommand::SetEffect(id) => assert_eq!(id, 5),
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_frame_data_serialization() {
        let frame = FrameData {
            width: 64,
            height: 64,
            format: FrameFormat::RGB,
            data: vec![255; 64 * 64 * 3],
        };

        let payload = frame.to_payload();
        let decoded = FrameData::from_payload(&payload).unwrap();

        assert_eq!(frame.width, decoded.width);
        assert_eq!(frame.height, decoded.height);
        assert_eq!(frame.data.len(), decoded.data.len());
    }
}
