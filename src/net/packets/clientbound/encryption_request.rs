use rsa::pkcs8::EncodePublicKey;

use crate::{RSA_PUBLIC_KEY, net::codec::write_var};

pub async fn send_encryption_request<W: tokio::io::AsyncWriteExt + Unpin>(stream: &mut W) -> anyhow::Result<()> {
    let mut packet_data = vec![0x01];

    write_var(&mut packet_data, "".len() as i32)?;
    packet_data.extend_from_slice("".as_bytes());

    let pkcs8 = RSA_PUBLIC_KEY.to_public_key_der()?;
    write_var(&mut packet_data, pkcs8.as_bytes().len() as i32)?;
    packet_data.extend_from_slice(pkcs8.as_bytes());

    
    let mut verify_token = [0u8; 4];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut verify_token);

    write_var(&mut packet_data, verify_token.len() as i32)?;
    packet_data.extend_from_slice(&verify_token);

    let should_authenticate = crate::config::SERVER_CONFIG.online_mode;
    packet_data.extend_from_slice(&[should_authenticate as u8]);

    let mut len_prefix = Vec::with_capacity(5);
    write_var(&mut len_prefix, packet_data.len() as i32)?;

    stream.write_all(&len_prefix).await?;
    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}