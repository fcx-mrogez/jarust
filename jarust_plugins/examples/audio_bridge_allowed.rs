use jarust::jaconfig::JaConfig;
use jarust::jaconfig::TransportType;
use jarust_plugins::audio_bridge::jahandle_ext::AudioBridge;
use jarust_plugins::audio_bridge::messages::AllowAction;
use jarust_plugins::audio_bridge::messages::AllowedMsg;
use jarust_plugins::audio_bridge::messages::CreateRoomMsg;
use std::path::Path;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let filename = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    let env_filter = EnvFilter::from_default_env()
        .add_directive("jarust=trace".parse()?)
        .add_directive(format!("{filename}=trace").parse()?);
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let timeout = std::time::Duration::from_secs(10);

    let config = JaConfig::builder().url("ws://localhost:8188/ws").build();
    let mut connection = jarust::connect(config, TransportType::Ws).await?;
    let session = connection.create(10).await?;
    let (handle, ..) = session.attach_audio_bridge().await?;

    let create_room_rsp = handle
        .create_room_with_config(
            CreateRoomMsg {
                secret: Some("superdupersecret".to_string()),
                ..Default::default()
            },
            timeout,
        )
        .await?;
    tracing::info!(
        "Created Room {}, permanent: {}",
        create_room_rsp.room,
        create_room_rsp.permanent
    );

    let allowed_rsp = handle
        .allowed(
            create_room_rsp.room,
            AllowedMsg {
                action: AllowAction::Add,
                allowed: vec![],
                secret: Some("superdupersecret".to_string()),
            },
            timeout,
        )
        .await?;

    tracing::info!(
        "Allowed participants in room {}: {:#?}",
        allowed_rsp.room,
        allowed_rsp.allowed
    );

    Ok(())
}
