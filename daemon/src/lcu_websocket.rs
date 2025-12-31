//! LCU WebSocket client for real-time events from the League Client.
//!
//! This provides a more efficient alternative to polling for events like:
//! - Gameflow phase changes (lobby, champ select, in game, etc.)
//! - Queue pop notifications
//! - Match found events
//! - End of game stats

use crate::{AppError, Result};
use crate::LcuConnection;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{
        client::IntoClientRequest,
        http::header::{AUTHORIZATION, HeaderValue},
        Message,
    },
    Connector,
};
use tracing::{debug, error, info, warn};

/// LCU WebSocket event types we care about
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LcuEvent {
    /// The event URI (e.g., "/lol-gameflow/v1/gameflow-phase")
    pub uri: String,
    /// The event type (Create, Update, Delete)
    pub event_type: String,
    /// The event data (JSON value)
    pub data: serde_json::Value,
}

/// Subscriptions for LCU WebSocket events
#[derive(Debug, Clone, Copy)]
pub enum LcuSubscription {
    /// All JSON API events
    JsonApiEvent,
    /// Specific endpoint events
    JsonApiEventPrefix(&'static str),
}

impl LcuSubscription {
    fn as_subscription_string(&self) -> String {
        match self {
            LcuSubscription::JsonApiEvent => "OnJsonApiEvent".to_string(),
            LcuSubscription::JsonApiEventPrefix(prefix) => {
                format!("OnJsonApiEvent_{}", prefix.replace('/', "_"))
            }
        }
    }
}

/// LCU WebSocket client for receiving real-time events
pub struct LcuWebSocket {
    /// Channel to receive events
    event_rx: mpsc::Receiver<LcuEvent>,
    /// Handle to the WebSocket task
    _task_handle: tokio::task::JoinHandle<()>,
}

impl LcuWebSocket {
    /// Connect to the LCU WebSocket and start receiving events.
    /// Automatically discovers the LCU connection from the lockfile.
    pub async fn connect() -> Result<Self> {
        let connection = LcuConnection::from_lockfile()?;
        Self::connect_with(connection).await
    }

    /// Connect to the LCU WebSocket with provided credentials.
    pub async fn connect_with(connection: LcuConnection) -> Result<Self> {
        let url = format!("wss://127.0.0.1:{}", connection.port);
        info!("Connecting to LCU WebSocket at {}", url);

        // Build the request with auth header
        let mut request = url.into_client_request()
            .map_err(|e| AppError::Other(format!("Failed to create WebSocket request: {}", e)))?;

        let auth_value = {
            let credentials = format!("riot:{}", connection.auth_token);
            format!("Basic {}", BASE64.encode(credentials.as_bytes()))
        };
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value)
                .map_err(|e| AppError::Other(format!("Invalid auth header: {}", e)))?,
        );

        // Configure TLS to accept the LCU's self-signed certificate
        let tls_config = Self::create_tls_config()?;
        let connector = Connector::Rustls(Arc::new(tls_config));

        // Connect to the WebSocket
        let (ws_stream, _response) = connect_async_tls_with_config(request, None, false, Some(connector))
            .await
            .map_err(|e| AppError::Other(format!("WebSocket connection failed: {}", e)))?;

        info!("LCU WebSocket connected");

        let (mut write, mut read) = ws_stream.split();

        // Create event channel
        let (event_tx, event_rx) = mpsc::channel::<LcuEvent>(100);

        // Subscribe to all JSON API events
        let sub_msg = format!(r#"[5, "OnJsonApiEvent"]"#);
        write.send(Message::Text(sub_msg.into()))
            .await
            .map_err(|e| AppError::Other(format!("Failed to subscribe: {}", e)))?;

        info!("Subscribed to LCU events");

        // Spawn task to handle incoming messages
        let task_handle = tokio::spawn(async move {
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        if let Some(event) = Self::parse_event(&text) {
                            if event_tx.send(event).await.is_err() {
                                debug!("Event receiver dropped, stopping WebSocket");
                                break;
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("LCU WebSocket closed by server");
                        break;
                    }
                    Ok(_) => {} // Ignore ping/pong/binary
                    Err(e) => {
                        warn!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
            info!("LCU WebSocket task ended");
        });

        Ok(Self {
            event_rx,
            _task_handle: task_handle,
        })
    }

    /// Create TLS config that accepts the LCU's self-signed certificate
    fn create_tls_config() -> Result<rustls::ClientConfig> {
        use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
        use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
        use rustls::DigitallySignedStruct;

        /// Custom certificate verifier that accepts any certificate.
        /// This is necessary because the LCU uses a self-signed certificate.
        #[derive(Debug)]
        struct AcceptAnyCert;

        impl ServerCertVerifier for AcceptAnyCert {
            fn verify_server_cert(
                &self,
                _end_entity: &CertificateDer<'_>,
                _intermediates: &[CertificateDer<'_>],
                _server_name: &ServerName<'_>,
                _ocsp_response: &[u8],
                _now: UnixTime,
            ) -> std::result::Result<ServerCertVerified, rustls::Error> {
                Ok(ServerCertVerified::assertion())
            }

            fn verify_tls12_signature(
                &self,
                _message: &[u8],
                _cert: &CertificateDer<'_>,
                _dss: &DigitallySignedStruct,
            ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
                Ok(HandshakeSignatureValid::assertion())
            }

            fn verify_tls13_signature(
                &self,
                _message: &[u8],
                _cert: &CertificateDer<'_>,
                _dss: &DigitallySignedStruct,
            ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
                Ok(HandshakeSignatureValid::assertion())
            }

            fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
                vec![
                    rustls::SignatureScheme::RSA_PKCS1_SHA256,
                    rustls::SignatureScheme::RSA_PKCS1_SHA384,
                    rustls::SignatureScheme::RSA_PKCS1_SHA512,
                    rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
                    rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
                    rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
                    rustls::SignatureScheme::RSA_PSS_SHA256,
                    rustls::SignatureScheme::RSA_PSS_SHA384,
                    rustls::SignatureScheme::RSA_PSS_SHA512,
                    rustls::SignatureScheme::ED25519,
                ]
            }
        }

        let config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(AcceptAnyCert))
            .with_no_client_auth();

        Ok(config)
    }

    /// Parse a WebSocket message into an LcuEvent.
    /// LCU sends messages in the format: [opcode, event_name, data]
    /// Opcode 8 = event message
    fn parse_event(text: &str) -> Option<LcuEvent> {
        let parsed: serde_json::Value = serde_json::from_str(text).ok()?;
        let arr = parsed.as_array()?;

        // Event messages have opcode 8
        let opcode = arr.first()?.as_u64()?;
        if opcode != 8 {
            return None;
        }

        // Get the event name and data
        let event_name = arr.get(1)?.as_str()?;
        let data = arr.get(2)?;

        // Extract the event details
        let event_data = data.get("data")?;
        let uri = data.get("uri")?.as_str()?;
        let event_type = data.get("eventType")?.as_str()?;

        Some(LcuEvent {
            uri: uri.to_string(),
            event_type: event_type.to_string(),
            data: event_data.clone(),
        })
    }

    /// Receive the next event from the WebSocket.
    /// Returns None if the connection is closed.
    pub async fn recv(&mut self) -> Option<LcuEvent> {
        self.event_rx.recv().await
    }

    /// Try to receive an event without blocking.
    pub fn try_recv(&mut self) -> Option<LcuEvent> {
        self.event_rx.try_recv().ok()
    }
}

/// Common LCU event URIs
pub mod uris {
    /// Gameflow phase changes (lobby, champ select, in game, etc.)
    pub const GAMEFLOW_PHASE: &str = "/lol-gameflow/v1/gameflow-phase";
    /// Current session info
    pub const GAMEFLOW_SESSION: &str = "/lol-gameflow/v1/session";
    /// Matchmaking search state
    pub const MATCHMAKING_SEARCH: &str = "/lol-matchmaking/v1/search";
    /// Champ select session
    pub const CHAMP_SELECT_SESSION: &str = "/lol-champ-select/v1/session";
    /// End of game stats
    pub const EOG_STATS: &str = "/lol-end-of-game/v1/eog-stats-block";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_event() {
        let msg = r#"[8,"OnJsonApiEvent",{"data":"InProgress","eventType":"Update","uri":"/lol-gameflow/v1/gameflow-phase"}]"#;
        let event = LcuWebSocket::parse_event(msg).unwrap();
        assert_eq!(event.uri, "/lol-gameflow/v1/gameflow-phase");
        assert_eq!(event.event_type, "Update");
        assert_eq!(event.data, serde_json::json!("InProgress"));
    }
}
