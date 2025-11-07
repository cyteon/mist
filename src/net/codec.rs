use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::net::packet::{self, HandshakePacket};

pub async fn read_var<R: AsyncReadExt + Unpin>(reader: &mut R) -> anyhow::Result<u32> {
    let mut num_read = 0;
    let mut result = 0;

    loop {
        let mut buf = [0];

        reader.read_exact(&mut buf).await?;
        let byte = buf[0];

        result |= ((byte & 0x7F) as u32) << (7 * num_read);
        num_read += 1;

        if num_read > 5 {
            return Err(anyhow::anyhow!("VarInt is too big"));
        }

        if (byte & 0x80) == 0 {
            break;
        }
    }

    Ok(result)
}

pub async fn write_var<W: AsyncWriteExt + Unpin>(stream: &mut W, mut value: i32) -> anyhow::Result<()> {
    loop {
        let mut temp = (value & 0b01111111) as u8;

        value >>= 7;

        if value != 0 {
            temp |= 0b10000000;
        }

        stream.write_u8(temp).await?;

        if value == 0 {
            break;
        }
    }
    Ok(())
}