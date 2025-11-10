use tokio::io::AsyncReadExt;

pub async fn read_known_packs<R: AsyncReadExt + Unpin>(stream: &mut R) -> anyhow::Result<()> {
    // todo: read and use this data for smth

    Ok(())
}