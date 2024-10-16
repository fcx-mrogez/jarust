use jarust::jaconfig::JaConfig;
use jarust::jaconfig::JanusAPI;
use jarust::jaconnection::CreateConnectionParams;
use jarust_plugins::audio_bridge::jahandle_ext::AudioBridge;
use jarust_plugins::audio_bridge::msg_opitons::AudioBridgeMuteOptions;
use jarust_transport::tgenerator::RandomTransactionGenerator;
use std::path::Path;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let filename = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    let env_filter = EnvFilter::from_default_env()
        .add_directive("jarust=trace".parse()?)
        .add_directive("jarust_plugins=trace".parse()?)
        .add_directive("jarust_transport=trace".parse()?)
        .add_directive("jarust_rt=trace".parse()?)
        .add_directive(format!("{filename}=trace").parse()?);
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let timeout = std::time::Duration::from_secs(10);
    let config = JaConfig::builder()
        .url("ws://localhost:8188/ws")
        .capacity(
            32, /* Buffer size on the entire connection with janus */
        )
        .build();
    let mut connection =
        jarust::connect(config, JanusAPI::WebSocket, RandomTransactionGenerator).await?;
    let session = connection
        .create_session(CreateConnectionParams {
            ka_interval: 10,
            timeout,
        })
        .await?;
    let (handle, mut events) = session.attach_audio_bridge(timeout).await?;

    let create_room_rsp = handle.create_room(None, timeout).await?;
    let rooms = handle.list_rooms(timeout).await?;

    tracing::info!("Rooms {:#?}", rooms);

    handle
        .join_room(
            create_room_rsp.room.clone(),
            Default::default(),
            None,
            timeout,
        )
        .await?;

    let list_participants_rsp = handle
        .list_participants(create_room_rsp.room, timeout)
        .await?;
    tracing::info!(
        "Participants in room {:#?}: {:#?}",
        list_participants_rsp.room,
        list_participants_rsp.participants
    );

    use jarust_plugins::audio_bridge::events::AudioBridgeEvent as ABE;
    use jarust_plugins::audio_bridge::events::PluginEvent as PE;
    if let Some(PE::AudioBridgeEvent(ABE::RoomJoined { id, room, .. })) = events.recv().await {
        handle
            .mute(AudioBridgeMuteOptions {
                id: id.clone(),
                room: room.clone(),
                secret: None,
            })
            .await?;

        handle
            .unmute(AudioBridgeMuteOptions {
                id,
                room,
                secret: None,
            })
            .await?;
    };

    while let Some(e) = events.recv().await {
        tracing::info!("{e:#?}");
    }

    Ok(())
}
