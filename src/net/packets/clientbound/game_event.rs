use tokio::io::AsyncWriteExt;

pub enum GameEvent {
    //NoRespawnBlockAvailable = 0,
    //StartRaining = 1,
    //EndRaining = 2,
    ChangeGameMode = 3,
    //WinGame = 4,
    //DemoEvent = 5,
    //ArrowHitPlayer = 6,
    //FadeValueChanged = 7,
    //FadeTimeChanged = 8,
    //MobAppearance = 10,
    //EnableRespawnScreen = 11,
    //LimitedCrafting = 12,
    //StartWaitingForLevelChunks = 13
}

pub async fn send_game_event<W: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut W,
    event: u8,
    value: f32,
) -> anyhow::Result<()> {
    let mut packet_data = vec![crate::net::packet::play::clientbound::GAME_EVENT as u8];

    packet_data.push(event);
    packet_data.write_f32(value).await?;

    stream.write_all(&packet_data).await?;
    stream.flush().await?;

    Ok(())
}
