use std::io::Read;

/// ```
///       0        1        2        3        4        5        6        7
///       +--------+--------+--------+--------+--------+--------+--------+--------+
/// 0x00  |                               Identifier                              |
///       +--------+--------+--------+--------+--------+--------+--------+--------+
/// 0x08  |                                paths_len                              |
///       +--------+--------+--------+--------+--------+--------+--------+--------+
/// 0x10  |                                sizes_len                              |
///       +--------+--------+--------+--------+--------+--------+--------+--------+
/// 0x18  |                                hashes_len                             |
///       +--------+--------+--------+--------+--------+--------+--------+--------+
/// 0x20  |                          compressed_data_len                          |
///       +--------+--------+--------+--------+--------+--------+--------+--------+
/// ```
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Header {
    pub paths_len: u64,
    pub sizes_len: u64,
    pub hashes_len: u64,
    pub compressed_data_len: u64,
}

impl Header {
    pub const IDENTIFIER: [u8; 8] = u64::to_le_bytes(0xA0B0C0D0A1B1C1D1);
    pub const HEADER_LEN: u64 = 40;

    pub fn parse_stream(reader: &mut impl Read) -> Result<Self, std::io::Error> {
        let mut buf = [0u8; 8];

        reader.read_exact(&mut buf)?;
        if buf != Header::IDENTIFIER {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid header indentifier",
            ));
        }
        reader.read_exact(&mut buf)?;
        let paths_len = u64::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let sizes_len = u64::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let hashes_len = u64::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let compressed_data_len = u64::from_le_bytes(buf);

        Ok(Self {
            paths_len,
            sizes_len,
            hashes_len,
            compressed_data_len,
        })
    }

    pub fn to_bytes(&self) -> [u8; 40] {
        let mut buf = [0u8; 40];
        let mut ptr = 0;

        buf[ptr..ptr + 8].copy_from_slice(&Self::IDENTIFIER);

        ptr += 8;
        buf[ptr..ptr + 8].copy_from_slice(&self.paths_len.to_le_bytes());

        ptr += 8;
        buf[ptr..ptr + 8].copy_from_slice(&self.sizes_len.to_le_bytes());

        ptr += 8;
        buf[ptr..ptr + 8].copy_from_slice(&self.hashes_len.to_le_bytes());

        ptr += 8;
        buf[ptr..ptr + 8].copy_from_slice(&self.compressed_data_len.to_le_bytes());

        buf
    }

    pub fn header_and_data_len(&self) -> u64 {
        Self::HEADER_LEN
            + self.paths_len
            + self.sizes_len
            + self.hashes_len
            + self.compressed_data_len
    }
}
