use rsa::Pkcs1v15Encrypt;
use tokio::io::AsyncReadExt;

use crate::{RSA_PRIVATE_KEY, net::codec::read_var};

pub struct EncryptionResponsePacket {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}

pub async fn read_encryption_response<R: AsyncReadExt + Unpin>(stream: &mut R) -> anyhow::Result<EncryptionResponsePacket> {
    let _packet_len = read_var(stream).await?;
    let packet_id = read_var(stream).await?;

    if packet_id != 0x01 {
        anyhow::bail!("Expected encryption response packet ID 0x01, got {}", packet_id);
    }

    let shared_secret_len = read_var(stream).await? as usize;
    let mut encrypted_shared_secret = vec![0u8; shared_secret_len];
    stream.read_exact(&mut encrypted_shared_secret).await?;

    let verify_token_len = read_var(stream).await? as usize;
    let mut verify_token = vec![0u8; verify_token_len];
    stream.read_exact(&mut verify_token).await?;

    let shared_secret = RSA_PRIVATE_KEY.decrypt(Pkcs1v15Encrypt, &encrypted_shared_secret)
        .map_err(|e| anyhow::anyhow!("Failed to decrypt shared secret: {}", e))?;

    Ok(EncryptionResponsePacket {
        shared_secret,
        verify_token,
    })
}