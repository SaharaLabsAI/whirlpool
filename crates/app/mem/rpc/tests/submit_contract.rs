use std::sync::{Arc, Mutex};

use app::traits::TxSource;
use rpc_mem::{MemoryTxService, SubmitPersonalityRequest, TxSourceMemoryTxService};

#[derive(Debug, Default)]
struct RecordingTxSource {
    pushed: Mutex<Vec<Vec<u8>>>,
}

impl RecordingTxSource {
    fn pushed(&self) -> Vec<Vec<u8>> {
        self.pushed
            .lock()
            .expect("poisoned tx source mutex")
            .clone()
    }
}

impl TxSource for RecordingTxSource {
    fn push(&self, tx: Vec<u8>) {
        self.pushed
            .lock()
            .expect("poisoned tx source mutex")
            .push(tx);
    }

    fn pending(&self) -> Vec<Vec<u8>> {
        self.pushed()
    }
}

fn sample_request() -> SubmitPersonalityRequest {
    SubmitPersonalityRequest {
        version: 1,
        signer: "0x7369676e65722d31".to_string(),
        personality_id: "0x706572736f6e612d31".to_string(),
        nonce: 7,
        markdown: "# Persona\nBe precise.".to_string(),
        signature_scheme: "raw_secp256k1".to_string(),
        signature: format!("0x{}", "11".repeat(65)),
    }
}

#[test]
fn submit_personality_enqueues_bytes_and_returns_hash() {
    let tx_source = Arc::new(RecordingTxSource::default());
    let service = TxSourceMemoryTxService::new(tx_source.clone());
    let request = sample_request();

    let tx_hash = service
        .submit_personality(request)
        .expect("valid request should be accepted");

    let pushed = tx_source.pushed();
    assert_eq!(
        pushed.len(),
        1,
        "accepted request should enqueue one payload"
    );
    assert_eq!(tx_hash.len(), 32);
    assert!(tx_hash.iter().any(|byte| *byte != 0));
}

#[test]
fn oversize_markdown_is_rejected_without_enqueue() {
    let tx_source = Arc::new(RecordingTxSource::default());
    let service = TxSourceMemoryTxService::new(tx_source.clone());
    let mut request = sample_request();
    request.markdown = "a".repeat(app_mem::MAX_PERSONALITY_MARKDOWN_BYTES + 1);

    let err = service
        .submit_personality(request)
        .expect_err("oversize markdown must be rejected");

    assert!(err.to_string().contains("markdown length"));
    assert!(
        tx_source.pushed().is_empty(),
        "rejected request must not enqueue bytes"
    );
}
