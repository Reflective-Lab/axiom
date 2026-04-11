// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Idiomatic Rust client for the Converge remote protocol.
//!
//! `converge-client` is intentionally thin. The wire contract lives in
//! `converge-protocol`; this crate adds a stable connection surface and typed
//! request envelopes for Rust consumers.

use tonic::transport::{Channel, Endpoint};

pub use converge_protocol as protocol;
pub use converge_protocol::{prost_types, v1};
pub use tonic::Streaming;
pub use tonic::transport::{Channel as TransportChannel, Endpoint as TransportEndpoint};

/// The streamed server event type returned by the Converge stream RPC.
pub type EventStream = Streaming<v1::ServerEvent>;

/// The generated tonic client used underneath the stable SDK wrapper.
pub type RawConvergeClient = v1::converge_service_client::ConvergeServiceClient<Channel>;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("invalid endpoint: {0}")]
    InvalidEndpoint(String),
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
    #[error(transparent)]
    Status(#[from] tonic::Status),
}

/// Stable Rust SDK for talking to a remote Converge runtime.
#[derive(Debug, Clone)]
pub struct ConvergeClient {
    inner: RawConvergeClient,
}

impl ConvergeClient {
    /// Connect to a remote Converge endpoint.
    pub async fn connect(uri: impl Into<String>) -> Result<Self, ClientError> {
        let endpoint = Endpoint::from_shared(uri.into())
            .map_err(|err| ClientError::InvalidEndpoint(err.to_string()))?;
        Self::connect_endpoint(endpoint).await
    }

    /// Connect using a pre-configured tonic endpoint.
    pub async fn connect_endpoint(endpoint: Endpoint) -> Result<Self, ClientError> {
        let inner = RawConvergeClient::connect(endpoint).await?;
        Ok(Self { inner })
    }

    /// Build a client from an existing tonic channel.
    pub fn from_channel(channel: Channel) -> Self {
        Self {
            inner: RawConvergeClient::new(channel),
        }
    }

    /// Access the underlying generated tonic client.
    pub fn inner(&self) -> &RawConvergeClient {
        &self.inner
    }

    /// Mutably access the underlying generated tonic client.
    pub fn inner_mut(&mut self) -> &mut RawConvergeClient {
        &mut self.inner
    }

    /// Consume the wrapper and return the underlying generated tonic client.
    pub fn into_inner(self) -> RawConvergeClient {
        self.inner
    }

    pub async fn submit_job(
        &mut self,
        request: v1::SubmitJobRequest,
    ) -> Result<v1::SubmitJobResponse, ClientError> {
        Ok(self.inner.submit_job(request).await?.into_inner())
    }

    pub async fn get_job(
        &mut self,
        request: v1::GetJobRequest,
    ) -> Result<v1::GetJobResponse, ClientError> {
        Ok(self.inner.get_job(request).await?.into_inner())
    }

    pub async fn get_events(
        &mut self,
        request: v1::GetEventsRequest,
    ) -> Result<v1::GetEventsResponse, ClientError> {
        Ok(self.inner.get_events(request).await?.into_inner())
    }

    pub async fn get_capabilities(
        &mut self,
        request: v1::GetCapabilitiesRequest,
    ) -> Result<v1::GetCapabilitiesResponse, ClientError> {
        Ok(self.inner.get_capabilities(request).await?.into_inner())
    }

    pub async fn stream<S>(&mut self, request: S) -> Result<EventStream, ClientError>
    where
        S: tonic::IntoStreamingRequest<Message = v1::ClientMessage>,
    {
        Ok(self.inner.stream(request).await?.into_inner())
    }
}

/// Helpers for wrapping typed requests into stream `ClientMessage` envelopes.
pub mod messages {
    use super::v1::{
        ApproveProposalRequest, CancelJobRequest, ClientMessage, Ping, PauseRunRequest,
        RejectProposalRequest, ResumeFromSequenceRequest, ResumeRunRequest, SubmitJobRequest,
        SubmitObservationRequest, SubscribeRequest, UnsubscribeRequest, UpdateBudgetRequest,
        client_message,
    };

    pub fn submit_job(request_id: impl Into<String>, request: SubmitJobRequest) -> ClientMessage {
        envelope(request_id, client_message::Message::SubmitJob(request))
    }

    pub fn cancel_job(request_id: impl Into<String>, request: CancelJobRequest) -> ClientMessage {
        envelope(request_id, client_message::Message::CancelJob(request))
    }

    pub fn submit_observation(
        request_id: impl Into<String>,
        request: SubmitObservationRequest,
    ) -> ClientMessage {
        envelope(
            request_id,
            client_message::Message::SubmitObservation(request),
        )
    }

    pub fn approve(
        request_id: impl Into<String>,
        request: ApproveProposalRequest,
    ) -> ClientMessage {
        envelope(request_id, client_message::Message::Approve(request))
    }

    pub fn reject(
        request_id: impl Into<String>,
        request: RejectProposalRequest,
    ) -> ClientMessage {
        envelope(request_id, client_message::Message::Reject(request))
    }

    pub fn pause(request_id: impl Into<String>, request: PauseRunRequest) -> ClientMessage {
        envelope(request_id, client_message::Message::Pause(request))
    }

    pub fn resume(request_id: impl Into<String>, request: ResumeRunRequest) -> ClientMessage {
        envelope(request_id, client_message::Message::Resume(request))
    }

    pub fn update_budget(
        request_id: impl Into<String>,
        request: UpdateBudgetRequest,
    ) -> ClientMessage {
        envelope(request_id, client_message::Message::UpdateBudget(request))
    }

    pub fn subscribe(request_id: impl Into<String>, request: SubscribeRequest) -> ClientMessage {
        envelope(request_id, client_message::Message::Subscribe(request))
    }

    pub fn unsubscribe(
        request_id: impl Into<String>,
        request: UnsubscribeRequest,
    ) -> ClientMessage {
        envelope(request_id, client_message::Message::Unsubscribe(request))
    }

    pub fn resume_from(
        request_id: impl Into<String>,
        request: ResumeFromSequenceRequest,
    ) -> ClientMessage {
        envelope(request_id, client_message::Message::ResumeFrom(request))
    }

    pub fn ping(request_id: impl Into<String>, request: Ping) -> ClientMessage {
        envelope(request_id, client_message::Message::Ping(request))
    }

    fn envelope(
        request_id: impl Into<String>,
        message: client_message::Message,
    ) -> ClientMessage {
        ClientMessage {
            request_id: request_id.into(),
            message: Some(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::messages;
    use super::v1::{PauseRunRequest, SubscribeRequest, client_message};

    #[test]
    fn pause_message_wraps_the_expected_variant() {
        let message = messages::pause(
            "pause-1",
            PauseRunRequest {
                run_id: "run-123".to_string(),
                reason: Some("operator requested pause".to_string()),
            },
        );

        assert_eq!(message.request_id, "pause-1");
        match message.message {
            Some(client_message::Message::Pause(request)) => {
                assert_eq!(request.run_id, "run-123");
            }
            other => panic!("unexpected message variant: {other:?}"),
        }
    }

    #[test]
    fn subscribe_message_wraps_the_expected_variant() {
        let message = messages::subscribe(
            "sub-1",
            SubscribeRequest {
                job_id: None,
                run_id: Some("run-123".to_string()),
                correlation_id: None,
                since_sequence: 42,
                entry_types: vec![],
            },
        );

        assert_eq!(message.request_id, "sub-1");
        match message.message {
            Some(client_message::Message::Subscribe(request)) => {
                assert_eq!(request.run_id.as_deref(), Some("run-123"));
                assert_eq!(request.since_sequence, 42);
            }
            other => panic!("unexpected message variant: {other:?}"),
        }
    }
}
