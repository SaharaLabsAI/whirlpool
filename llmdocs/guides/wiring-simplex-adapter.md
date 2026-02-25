# Wiring the Simplex Adapter

The Simplex Adapter Bridge requires several components to work together. This guide explains how to define your block type, set up the adapter, and initialize the engine.

## 1. Define your Block type

Your block must implement both the internal `CoreBlock` trait and the vendor's `VendorBlock` trait. It also needs to implement several codec and cryptography traits from the Commonware ecosystem. A simple approach is ensuring your type satisfies the bounds of the `CommonwareBlock` trait.

### Example: TestBlock Reference

The following `TestBlock` implementation from the test suite demonstrates the required trait implementations.

```rust
#[derive(Clone, Debug)]
struct TestBlock {
    id: [u8; 32],
    parent: TestDigest,
    height: u64,
}

// Internal CoreBlock implementation
impl CoreBlock for TestBlock {
    type Id = [u8; 32];

    fn id(&self) -> Self::Id {
        self.id
    }

    fn parent_id(&self) -> Self::Id {
        let commitment = self.parent;
        let bytes: &[u8] = commitment.as_ref();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes[..32]);
        arr
    }

    fn height(&self) -> u64 {
        self.height
    }
}

// Vendor Block implementation
impl VendorBlock for TestBlock {
    fn parent(&self) -> Self::Commitment {
        self.parent
    }
}

// Required commonware-consensus traits
impl Heightable for TestBlock {
    fn height(&self) -> commonware_consensus::types::Height {
        commonware_consensus::types::Height::new(self.height)
    }
}

// Required commonware-cryptography traits
impl Digestible for TestBlock {
    type Digest = TestDigest;
    fn digest(&self) -> Self::Digest {
        TestDigest::from(self.id)
    }
}

impl Committable for TestBlock {
    type Commitment = TestDigest;
    fn commitment(&self) -> Self::Commitment {
        self.digest()
    }
}
```

## 2. Wire the AppAdapter

Once your block type is ready, you can create the `AppAdapter`. This component wraps your `ConsensusApp` and `EventSink`. It handles the translation between the vendor's application traits and your internal logic.

```rust
let app = Arc::new(MyConsensusApp::new());
let sink = Arc::new(MyEventSink::new());

let adapter = AppAdapter::new(app, sink);
```

The adapter automatically implements the `Application`, `VerifyingApplication`, and `Reporter` traits required by the Commonware stack.

## 3. Create the CommonwareEngine

The `CommonwareEngine` uses a starter closure to initialize the vendor stack. This closure receives shared atomic variables for tracking the engine's height and running status.

```rust
let engine = CommonwareEngine::new(|height: Arc<AtomicU64>, running: Arc<AtomicBool>| {
    // 1. Initialize vendor components (Network, Storage, etc.)
    // 2. Set up the Commonware Simplex stack using the AppAdapter
    // 3. Spawn the vendor event loop
    // 4. Return a shutdown closure and a JoinHandle for the task
    
    let shutdown = Box::new(move || {
        running.store(false, Ordering::SeqCst);
    });
    
    Ok((shutdown, handle))
});
```

The engine can then be started using the standard `ConsensusEngine` interface.

## 4. Configure CommonwareConfig

The `CommonwareConfig` struct holds the parameters for the Simplex protocol. You'll typically load these from a configuration file or define them as constants.

```rust
let config = CommonwareConfig {
    namespace: "my-blockchain".to_string(),
    leader_timeout: Duration::from_millis(500),
    notarization_timeout: Duration::from_millis(1000),
    nullify_retry: Duration::from_millis(200),
    activity_timeout: 10,
    skip_timeout: 5,
    mailbox_size: 128,
    replay_buffer: NonZeroUsize::new(64).unwrap(),
    write_buffer: NonZeroUsize::new(32).unwrap(),
    epoch: 1,
    fetch_timeout: Duration::from_secs(5),
    fetch_concurrent: 4,
};
```

These settings control the timing and buffer sizes for the underlying consensus protocol.
