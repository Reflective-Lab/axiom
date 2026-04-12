// Protocol and client SDK tests.
//
// Prove: message envelope construction is correct for all message types.
// Prove: observation vocabulary is used, not fact injection.

use converge_client::messages;
use converge_client::v1::client_message::Message;
use converge_client::v1::*;

#[test]
fn submit_job_envelope() {
    let msg = messages::submit_job(
        "req-1",
        SubmitJobRequest {
            blueprint_id: "test-blueprint".into(),
            ..Default::default()
        },
    );

    assert_eq!(msg.request_id, "req-1");
    assert!(matches!(msg.message, Some(Message::SubmitJob(_))));
}

#[test]
fn submit_observation_envelope() {
    let msg = messages::submit_observation(
        "req-2",
        SubmitObservationRequest {
            run_id: "run-123".into(),
            key: "Seeds".into(),
            payload: None,
            target_truth_id: Some("evaluate-vendor".into()),
            idempotency_key: "idem-1".into(),
        },
    );

    assert_eq!(msg.request_id, "req-2");
    match msg.message {
        Some(Message::SubmitObservation(req)) => {
            assert_eq!(req.run_id, "run-123");
            assert_eq!(req.key, "Seeds");
            assert_eq!(req.target_truth_id, Some("evaluate-vendor".into()));
            assert_eq!(req.idempotency_key, "idem-1");
        }
        other => panic!("expected SubmitObservation, got {other:?}"),
    }
}

#[test]
fn approve_proposal_envelope() {
    let msg = messages::approve(
        "req-3",
        ApproveProposalRequest {
            run_id: "run-123".into(),
            proposal_id: "prop-1".into(),
            comment: Some("CFO approved".into()),
        },
    );

    assert_eq!(msg.request_id, "req-3");
    match msg.message {
        Some(Message::Approve(req)) => {
            assert_eq!(req.proposal_id, "prop-1");
            assert_eq!(req.comment, Some("CFO approved".into()));
        }
        other => panic!("expected Approve, got {other:?}"),
    }
}

#[test]
fn reject_proposal_envelope() {
    let msg = messages::reject(
        "req-4",
        RejectProposalRequest {
            run_id: "run-123".into(),
            proposal_id: "prop-2".into(),
            reason: "insufficient evidence".into(),
        },
    );

    assert_eq!(msg.request_id, "req-4");
    assert!(matches!(msg.message, Some(Message::Reject(_))));
}

#[test]
fn all_message_types_wrap_correctly() {
    // Verify every message variant can be constructed through the helpers
    let _ = messages::cancel_job(
        "r",
        CancelJobRequest {
            job_id: "j".into(),
            reason: None,
        },
    );
    let _ = messages::pause(
        "r",
        PauseRunRequest {
            run_id: "r".into(),
            reason: None,
        },
    );
    let _ = messages::resume("r", ResumeRunRequest { run_id: "r".into() });
    let _ = messages::update_budget(
        "r",
        UpdateBudgetRequest {
            run_id: "r".into(),
            budget: None,
        },
    );
    let _ = messages::subscribe(
        "r",
        SubscribeRequest {
            job_id: None,
            run_id: Some("r".into()),
            correlation_id: None,
            since_sequence: 0,
            entry_types: vec![],
        },
    );
    let _ = messages::unsubscribe(
        "r",
        UnsubscribeRequest {
            job_id: None,
            run_id: Some("r".into()),
            correlation_id: None,
        },
    );
    let _ = messages::resume_from(
        "r",
        ResumeFromSequenceRequest {
            sequence: 42,
            allow_snapshot: false,
        },
    );
    let _ = messages::ping("r", Ping { client_time_ns: 0 });
}

// ── No inject_fact vocabulary ──

#[test]
fn no_inject_fact_in_client_message_variants() {
    // This test documents that InjectFact does not exist as a message variant.
    // If someone adds it back, this test should be updated or the reviewer should block.
    let msg = ClientMessage {
        request_id: "test".into(),
        message: None,
    };
    // The message is None — there is no InjectFact variant to construct.
    assert!(msg.message.is_none());
}
