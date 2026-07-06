use byteorder::{BigEndian, WriteBytesExt};
use tokio::io::AsyncReadExt;

use crate::types::items::ItemStack;

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

pub fn write_var<W: WriteBytesExt + Unpin>(stream: &mut W, value: i32) -> anyhow::Result<()> {
    let mut value = value as u32;

    loop {
        let mut temp = (value & 0b01111111) as u8;

        value >>= 7;

        if value != 0 {
            temp |= 0b10000000;
        }

        stream.write_u8(temp)?;

        if value == 0 {
            break;
        }
    }

    Ok(())
}

pub fn write_string<W: WriteBytesExt + Unpin>(stream: &mut W, value: &str) -> anyhow::Result<()> {
    write_var(stream, value.len() as i32)?;
    stream.write_all(value.as_bytes())?;

    Ok(())
}

pub fn write_slot<W: WriteBytesExt + Unpin>(
    stream: &mut W,
    item_stack: Option<ItemStack>,
) -> anyhow::Result<()> {
    if let Some(item_stack) = item_stack {
        write_var(stream, item_stack.count as i32)?;
        write_var(stream, item_stack.item_id)?;

        let components_data = crate::types::items::get_item_components(item_stack.item_id);

        write_var(stream, components_data.len() as i32)?;
        write_var(stream, 0)?;

        for (t, b) in components_data {
            write_var(stream, t)?;
            stream.write_all(&b)?;
        }
    } else {
        write_var(stream, 0)?;
    }

    Ok(())
}

// x: 26 bits z: 26 bits, y: 12 bits,
pub fn write_position<W: WriteBytesExt + Unpin>(
    stream: &mut W,
    x: i32,
    y: i32,
    z: i32,
) -> anyhow::Result<()> {
    let mut val = 0i64;

    val |= (x as i64 & 0x3FFFFFF) << 38;
    val |= (z as i64 & 0x3FFFFFF) << 12;
    val |= y as i64 & 0xFFF;

    stream.write_i64::<BigEndian>(val)?;

    Ok(())
}

// x: 26 bits z: 26 bits, y: 12 bits,
// all signed integers, two's complement
pub async fn read_position<R: AsyncReadExt + Unpin>(
    stream: &mut R,
) -> anyhow::Result<(i32, i32, i32)> {
    let val = stream.read_i64().await?;

    let x = (val >> 38) as i32;
    let z = (val >> 12 & 0x3FFFFFF) as i32;
    let y = (val & 0xFFF) as i32;

    let x = if x >= 0x2000000 { x - 0x4000000 } else { x };
    let z = if z >= 0x2000000 { z - 0x4000000 } else { z };

    let y = if y >= 0x800 { y - 0x1000 } else { y };

    Ok((x, y, z))
}

pub async fn read_slot<R: AsyncReadExt + Unpin>(
    stream: &mut R,
) -> anyhow::Result<Option<ItemStack>> {
    let count = read_var(stream).await? as u8;

    if count <= 0 {
        return Ok(None);
    }

    let item_id = read_var(stream).await? as i32;

    // TODO:
    // Number of components to add
    // Number of components to remove

    return Ok(Some(ItemStack {
        item_id,
        count: count,
    }));
}

pub async fn read_hashed_slot<R: AsyncReadExt + Unpin>(
    stream: &mut R,
) -> anyhow::Result<Option<ItemStack>> {
    let has_item = stream.read_u8().await?;

    if has_item == 0 {
        return Ok(None);
    }

    let item_id = read_var(stream).await? as i32;
    let count = read_var(stream).await? as u8;

    let components_to_add = read_var(stream).await?;
    for _ in 0..components_to_add {
        let _type = read_var(stream).await?;
        let _hash = stream.read_i32().await?;
    }

    let components_to_remove = read_var(stream).await?;
    for _ in 0..components_to_remove {
        let _type = read_var(stream).await?;
        let _hash = stream.read_i32().await?;
    }

    return Ok(Some(ItemStack {
        item_id,
        count: count,
    }));
}

pub fn normalize_angle(angle: f32) -> u8 {
    ((angle % 360.0 + 360.0) % 360.0 / 360.0 * 256.0) as u8
}

const MAX_VELOCITY: f64 = 1.717_986_918_3E10;
const MIN_MAGNITUDE: f64 = 3.051_944_088_384_301E-5;

pub fn write_lpvec3<W: WriteBytesExt + Unpin>(
    stream: &mut W,
    vx: f64,
    vy: f64,
    vz: f64,
) -> anyhow::Result<()> {
    let vx = vx.clamp(-MAX_VELOCITY, MAX_VELOCITY);
    let vy = vy.clamp(-MAX_VELOCITY, MAX_VELOCITY);
    let vz = vz.clamp(-MAX_VELOCITY, MAX_VELOCITY);
    let max = vx.abs().max(vy.abs()).max(vz.abs());

    if max < MIN_MAGNITUDE {
        stream.write_u8(0)?;
        return Ok(());
    }

    let scale_factor = max.ceil() as i64;

    let header = if scale_factor > 3 {
        (scale_factor & 3) | 4
    } else {
        scale_factor
    };

    let qx = to_long(vx / scale_factor as f64) << 3;
    let qy = to_long(vy / scale_factor as f64) << 18;
    let qz = to_long(vz / scale_factor as f64) << 33;

    let packed = header | qx | qy | qz;

    stream.write_all(&(packed as u16).to_le_bytes())?;
    stream.write_all(&((packed >> 16) as i32).to_be_bytes())?;

    if scale_factor > 3 {
        write_var(stream, (scale_factor >> 2) as i32)?;
    }

    Ok(())
}

fn to_long(value: f64) -> i64 {
    ((value.mul_add(0.5, 0.5) * 32766.0).round() as i64).clamp(0, 32766)
}
