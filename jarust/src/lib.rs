use crate::transport::trans::Transport;
use jaconfig::JaConfig;
use jaconnection::JaConnection;
use prelude::JaResult;

pub mod jaconfig;
pub mod jahandle;
pub mod japlugin;
pub mod japrotocol;
pub mod jasession;
pub mod prelude;
pub mod transport;

mod error;
mod jaconnection;
mod nsp_registry;
mod tmanager;
mod utils;

pub async fn connect(jaconfig: JaConfig) -> JaResult<JaConnection> {
    let transport = match jaconfig.transport_type {
        jaconfig::TransportType::Wss => transport::wss::WebsocketTransport::new(),
    };
    connect_with_transport(jaconfig, transport).await
}

pub async fn connect_with_transport(
    jaconfig: JaConfig,
    transport: impl Transport,
) -> JaResult<JaConnection> {
    log::info!("Creating new connection");
    log::trace!("Creating connection with server configuration {jaconfig:?}");
    JaConnection::open(jaconfig, transport).await
}
