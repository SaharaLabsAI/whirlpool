use super::*;
use commonware_cryptography::Signer;
use commonware_p2p::Ingress;
use commonware_formatting::hex;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_CONFIG_ID: AtomicU64 = AtomicU64::new(0);

fn validator_hexes(seeds: &[u64]) -> Vec<String> {
    seeds
        .iter()
        .map(|seed| hex(ed25519::PrivateKey::from_seed(*seed).public_key().as_ref()))
        .collect()
}

fn write_config_file(contents: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "whirlpool-node-test-config-{}.toml",
        NEXT_TEMP_CONFIG_ID.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&path, contents).expect("failed to write config file");
    path
}

#[test]
fn test_node_config_default_matches_hardcoded() {
    let config = NodeConfig::default();

    assert_eq!(config.network.namespace, b"whirlpool-dev");
    assert_eq!(config.network.listen_addr, BIND_ADDR.parse().unwrap());
    assert_eq!(config.network.dialable_addr, BIND_ADDR.parse().unwrap());
    assert!(config.network.bootstrap_peers.is_empty());
    assert_eq!(config.network.max_message_size, 1_048_576);
    assert_eq!(config.identity.seed, 0);
    assert_eq!(config.rpc.bind_addr, RPC_BIND_ADDR.parse().unwrap());
    assert_eq!(config.rpc.mem_bind_addr, MEM_RPC_BIND_ADDR.parse().unwrap());
    assert_eq!(config.storage.data_dir, PathBuf::from("data"));
    assert_eq!(config.consensus.namespace, b"sahara-chain-v0");
    assert_eq!(config.consensus.block_interval, Duration::from_secs(5));
    assert_eq!(config.bootstrap_validators, None);
    assert!(!config.bootstrap.genesis_bootstrap_dkg);
    assert_eq!(config.bootstrap.genesis_bootstrap_validator_count, None);
    assert_eq!(config.bootstrap.genesis_dkg_session_dir, None);
    assert_eq!(config.bootstrap.genesis_dkg_dealer_pubkey, None);
}

#[test]
fn test_storage_config_path_helpers() {
    let config = StorageConfig {
        data_dir: PathBuf::from("custom-data"),
    };

    assert_eq!(config.runtime_dir(), PathBuf::from("custom-data/runtime"));
    assert_eq!(config.state_dir(), PathBuf::from("custom-data/state"));
    assert_eq!(config.mempool_dir(), PathBuf::from("custom-data/mempool"));
}

#[test]
fn test_parse_bootstrap_peer_valid() {
    let public_key = ed25519::PrivateKey::from_seed(7).public_key();
    let encoded = format!("{}@127.0.0.1:3000", hex(public_key.as_ref()));

    let parsed = parse_bootstrap_peer(&encoded).expect("bootstrap peer should parse");

    assert_eq!(parsed.0, public_key);
    assert_eq!(parsed.1, Ingress::Socket("127.0.0.1:3000".parse().unwrap()));
}

#[test]
fn test_parse_bootstrap_peer_invalid_format() {
    let err = parse_bootstrap_peer("not-a-bootstrap-peer").expect_err("format should fail");

    assert!(err.contains("formatted as <pubkey>@<socket_addr>"));
}

#[test]
fn test_parse_bootstrap_peer_malformed_variants() {
    assert!(parse_bootstrap_peer("deadbeef127.0.0.1:3000").is_err());
    assert!(parse_bootstrap_peer("@127.0.0.1:3000").is_err());
    let pk = ed25519::PrivateKey::from_seed(1).public_key();
    let pk_hex = hex(pk.as_ref());
    assert!(parse_bootstrap_peer(&format!("{pk_hex}@")).is_err());
    assert!(parse_bootstrap_peer("ZZZZ@127.0.0.1:3000").is_err());
    assert!(parse_bootstrap_peer("aabb@127.0.0.1:3000").is_err());
    assert!(parse_bootstrap_peer(&format!("{pk_hex}@not-an-addr")).is_err());
}

#[test]
fn test_node_args_to_node_config_full_custom() {
    let pk1 = ed25519::PrivateKey::from_seed(10).public_key();
    let pk2 = ed25519::PrivateKey::from_seed(20).public_key();
    let peer1 = format!("{}@10.0.0.1:5000", hex(pk1.as_ref()));
    let peer2 = format!("{}@10.0.0.2:6000", hex(pk2.as_ref()));
    let validators = validator_hexes(&[10, 20]);

    let args = NodeArgs {
        config: None,
        listen_addr: Some("0.0.0.0:9000".parse().unwrap()),
        dialable_addr: Some("1.2.3.4:9000".parse().unwrap()),
        bootstrap_peer: vec![peer1],
        dial_peer: vec![peer2],
        validator_seed: Some(42),
        validator: validators.clone(),
        rpc_addr: Some("0.0.0.0:8080".parse().unwrap()),
        mem_rpc_addr: Some("0.0.0.0:8180".parse().unwrap()),
        data_dir: Some(PathBuf::from("/tmp/whirlpool")),
        max_message_size: Some(2_000_000),
        network_namespace: Some("custom-net".to_string()),
        consensus_namespace: Some("custom-cons".to_string()),
        block_interval_ms: Some(2000),
        genesis_bootstrap_dkg: false,
        genesis_bootstrap_validator_count: None,
        genesis_dkg_session_dir: None,
        genesis_dkg_dealer_pubkey: None,
    };

    let config = NodeConfig::from(args);

    assert_eq!(config.network.listen_addr, "0.0.0.0:9000".parse().unwrap());
    assert_eq!(
        config.network.dialable_addr,
        "1.2.3.4:9000".parse().unwrap()
    );
    assert_eq!(config.network.bootstrap_peers.len(), 2);
    assert_eq!(config.network.bootstrap_peers[0].0, pk1);
    assert_eq!(config.network.bootstrap_peers[1].0, pk2);
    assert_eq!(config.network.max_message_size, 2_000_000);
    assert_eq!(config.network.namespace, b"custom-net");
    assert_eq!(config.identity.seed, 42);
    assert_eq!(config.rpc.bind_addr, "0.0.0.0:8080".parse().unwrap());
    assert_eq!(config.rpc.mem_bind_addr, "0.0.0.0:8180".parse().unwrap());
    assert_eq!(config.storage.data_dir, PathBuf::from("/tmp/whirlpool"));
    assert_eq!(config.consensus.namespace, b"custom-cons");
    assert_eq!(config.consensus.block_interval, Duration::from_millis(2000));
    assert_eq!(config.bootstrap_validators.unwrap().len(), validators.len());
}

#[test]
fn test_node_args_peers_accumulate() {
    let pk1 = ed25519::PrivateKey::from_seed(30).public_key();
    let pk2 = ed25519::PrivateKey::from_seed(31).public_key();
    let pk3 = ed25519::PrivateKey::from_seed(32).public_key();
    let p1 = format!("{}@10.0.0.1:1111", hex(pk1.as_ref()));
    let p2 = format!("{}@10.0.0.2:2222", hex(pk2.as_ref()));
    let p3 = format!("{}@10.0.0.3:3333", hex(pk3.as_ref()));

    let args = NodeArgs::parse_from([
        "whirlpool-node",
        "--bootstrap-peer",
        &p1,
        "--bootstrap-peer",
        &p2,
        "--dial-peer",
        &p3,
    ]);

    let config = NodeConfig::from(args);

    assert_eq!(config.network.bootstrap_peers.len(), 3);
    assert_eq!(config.network.bootstrap_peers[0].0, pk1);
    assert_eq!(config.network.bootstrap_peers[1].0, pk2);
    assert_eq!(config.network.bootstrap_peers[2].0, pk3);
}

#[test]
fn test_node_args_default_roundtrip() {
    let args = NodeArgs::parse_from(["whirlpool-node"]);
    let from_args = NodeConfig::from(args);
    let from_default = NodeConfig::default();

    assert_eq!(from_args, from_default);
}

#[test]
fn tst_01_toml_file_loading() {
    let validator = validator_hexes(&[44]).pop().unwrap();
    let bootstrap_key = ed25519::PrivateKey::from_seed(45).public_key();
    let bootstrap_peer = format!("{}@127.0.0.1:4010", hex(bootstrap_key.as_ref()));
    let path = write_config_file(&format!(
        "listen_addr = \"127.0.0.1:4011\"\ndialable_addr = \"10.0.0.1:4011\"\nbootstrap_peers = [\"{bootstrap_peer}\"]\nvalidator_seed = 99\nrpc_addr = \"127.0.0.1:9555\"\nmem_rpc_addr = \"127.0.0.1:9655\"\ndata_dir = \"custom-data\"\nmax_message_size = 2097152\nnetwork_namespace = \"toml-net\"\nconsensus_namespace = \"toml-consensus\"\nblock_interval_ms = 1234\nbootstrap_validators = [\"{validator}\"]\n"
    ));

    let config = load_config(NodeArgs {
        config: Some(path),
        listen_addr: None,
        dialable_addr: None,
        bootstrap_peer: vec![],
        dial_peer: vec![],
        validator_seed: None,
        validator: vec![],
        rpc_addr: None,
        mem_rpc_addr: None,
        data_dir: None,
        max_message_size: None,
        network_namespace: None,
        consensus_namespace: None,
        block_interval_ms: None,
        genesis_bootstrap_dkg: false,
        genesis_bootstrap_validator_count: None,
        genesis_dkg_session_dir: None,
        genesis_dkg_dealer_pubkey: None,
    })
    .expect("toml config should load");

    assert_eq!(
        config.network.listen_addr,
        "127.0.0.1:4011".parse().unwrap()
    );
    assert_eq!(
        config.network.dialable_addr,
        "10.0.0.1:4011".parse().unwrap()
    );
    assert_eq!(config.network.bootstrap_peers.len(), 1);
    assert_eq!(config.identity.seed, 99);
    assert_eq!(config.rpc.bind_addr, "127.0.0.1:9555".parse().unwrap());
    assert_eq!(config.rpc.mem_bind_addr, "127.0.0.1:9655".parse().unwrap());
    assert_eq!(config.storage.data_dir, PathBuf::from("custom-data"));
    assert_eq!(config.network.max_message_size, 2_097_152);
    assert_eq!(config.network.namespace, b"toml-net");
    assert_eq!(config.consensus.namespace, b"toml-consensus");
    assert_eq!(config.consensus.block_interval, Duration::from_millis(1234));
    assert_eq!(config.bootstrap_validators.unwrap().len(), 1);
}

#[test]
fn tst_02_cli_overrides_toml() {
    let path = write_config_file(
        "listen_addr = \"127.0.0.1:4011\"\nrpc_addr = \"127.0.0.1:9555\"\nmem_rpc_addr = \"127.0.0.1:9655\"\nvalidator_seed = 7\nmax_message_size = 1000\nnetwork_namespace = \"toml-net\"\nbootstrap_validators = [\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"]\n",
    );
    let cli_validators = validator_hexes(&[2, 3]);

    let config = load_config(NodeArgs {
        config: Some(path),
        listen_addr: Some("0.0.0.0:5000".parse().unwrap()),
        dialable_addr: None,
        bootstrap_peer: vec![],
        dial_peer: vec![],
        validator_seed: Some(42),
        validator: cli_validators.clone(),
        rpc_addr: Some("0.0.0.0:8546".parse().unwrap()),
        mem_rpc_addr: Some("0.0.0.0:8646".parse().unwrap()),
        data_dir: None,
        max_message_size: Some(2048),
        network_namespace: Some("cli-net".into()),
        consensus_namespace: None,
        block_interval_ms: None,
        genesis_bootstrap_dkg: false,
        genesis_bootstrap_validator_count: None,
        genesis_dkg_session_dir: None,
        genesis_dkg_dealer_pubkey: None,
    })
    .expect("cli should override toml");

    assert_eq!(config.network.listen_addr, "0.0.0.0:5000".parse().unwrap());
    assert_eq!(config.rpc.bind_addr, "0.0.0.0:8546".parse().unwrap());
    assert_eq!(config.rpc.mem_bind_addr, "0.0.0.0:8646".parse().unwrap());
    assert_eq!(config.identity.seed, 42);
    assert_eq!(config.network.max_message_size, 2048);
    assert_eq!(config.network.namespace, b"cli-net");
    assert_eq!(
        config.bootstrap_validators.unwrap().len(),
        cli_validators.len()
    );
}

#[test]
fn tst_03_no_config_backward_compat() {
    let validator = validator_hexes(&[8]).pop().unwrap();
    let args = NodeArgs {
        config: None,
        listen_addr: Some("0.0.0.0:9000".parse().unwrap()),
        dialable_addr: Some("1.2.3.4:9000".parse().unwrap()),
        bootstrap_peer: vec![],
        dial_peer: vec![],
        validator_seed: Some(88),
        validator: vec![validator],
        rpc_addr: Some("0.0.0.0:8546".parse().unwrap()),
        mem_rpc_addr: Some("0.0.0.0:8646".parse().unwrap()),
        data_dir: Some(PathBuf::from("compat-data")),
        max_message_size: Some(4096),
        network_namespace: Some("compat-net".into()),
        consensus_namespace: Some("compat-cons".into()),
        block_interval_ms: Some(555),
        genesis_bootstrap_dkg: false,
        genesis_bootstrap_validator_count: None,
        genesis_dkg_session_dir: None,
        genesis_dkg_dealer_pubkey: None,
    };

    let expected = NodeConfig::from(args.clone());
    let actual = load_config(args).expect("load_config without file should match From<NodeArgs>");

    assert_eq!(actual, expected);
}

#[test]
fn tst_04_multi_validator_from_toml() {
    let validator_hexes = validator_hexes(&[1, 2, 3, 4]);
    let validator_list = validator_hexes
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let path = write_config_file(&format!("bootstrap_validators = [{validator_list}]\n"));

    let config = load_config(NodeArgs {
        config: Some(path),
        listen_addr: None,
        dialable_addr: None,
        bootstrap_peer: vec![],
        dial_peer: vec![],
        validator_seed: None,
        validator: vec![],
        rpc_addr: None,
        mem_rpc_addr: None,
        data_dir: None,
        max_message_size: None,
        network_namespace: None,
        consensus_namespace: None,
        block_interval_ms: None,
        genesis_bootstrap_dkg: false,
        genesis_bootstrap_validator_count: None,
        genesis_dkg_session_dir: None,
        genesis_dkg_dealer_pubkey: None,
    })
    .expect("multi-validator toml should parse");

    let validators = config
        .bootstrap_validators
        .expect("validators should exist");
    assert_eq!(validators.len(), 4);
    assert_eq!(
        validators[0],
        ed25519::PrivateKey::from_seed(1).public_key()
    );
    assert_eq!(
        validators[3],
        ed25519::PrivateKey::from_seed(4).public_key()
    );
}

#[test]
fn tst_08_missing_config_file_error() {
    let missing = std::env::temp_dir().join("whirlpool-node-missing-config.toml");
    let err = load_config(NodeArgs {
        config: Some(missing.clone()),
        listen_addr: None,
        dialable_addr: None,
        bootstrap_peer: vec![],
        dial_peer: vec![],
        validator_seed: None,
        validator: vec![],
        rpc_addr: None,
        mem_rpc_addr: None,
        data_dir: None,
        max_message_size: None,
        network_namespace: None,
        consensus_namespace: None,
        block_interval_ms: None,
        genesis_bootstrap_dkg: false,
        genesis_bootstrap_validator_count: None,
        genesis_dkg_session_dir: None,
        genesis_dkg_dealer_pubkey: None,
    })
    .expect_err("missing config should error");

    assert!(matches!(err, ConfigError::ReadConfig { ref path, .. } if path == &missing));
    assert!(err.to_string().contains("failed to read config file"));
}

#[test]
fn tst_09_invalid_toml_error() {
    let path = write_config_file("listen_addr = [\n");
    let err = load_config(NodeArgs {
        config: Some(path),
        listen_addr: None,
        dialable_addr: None,
        bootstrap_peer: vec![],
        dial_peer: vec![],
        validator_seed: None,
        validator: vec![],
        rpc_addr: None,
        mem_rpc_addr: None,
        data_dir: None,
        max_message_size: None,
        network_namespace: None,
        consensus_namespace: None,
        block_interval_ms: None,
        genesis_bootstrap_dkg: false,
        genesis_bootstrap_validator_count: None,
        genesis_dkg_session_dir: None,
        genesis_dkg_dealer_pubkey: None,
    })
    .expect_err("invalid toml should error");

    assert!(matches!(err, ConfigError::ParseToml { .. }));
    assert!(err.to_string().contains("failed to parse TOML config"));
}

#[test]
fn tst_10_partial_toml_cli_merge() {
    let path = write_config_file(
        "dialable_addr = \"10.0.0.2:5000\"\ndata_dir = \"toml-data\"\nconsensus_namespace = \"toml-cons\"\n",
    );

    let config = load_config(NodeArgs {
        config: Some(path),
        listen_addr: Some("0.0.0.0:5000".parse().unwrap()),
        dialable_addr: None,
        bootstrap_peer: vec![],
        dial_peer: vec![],
        validator_seed: None,
        validator: vec![],
        rpc_addr: None,
        mem_rpc_addr: None,
        data_dir: None,
        max_message_size: Some(8192),
        network_namespace: Some("cli-net".into()),
        consensus_namespace: None,
        block_interval_ms: Some(2500),
        genesis_bootstrap_dkg: false,
        genesis_bootstrap_validator_count: None,
        genesis_dkg_session_dir: None,
        genesis_dkg_dealer_pubkey: None,
    })
    .expect("partial merge should work");

    assert_eq!(config.network.listen_addr, "0.0.0.0:5000".parse().unwrap());
    assert_eq!(
        config.network.dialable_addr,
        "10.0.0.2:5000".parse().unwrap()
    );
    assert_eq!(config.storage.data_dir, PathBuf::from("toml-data"));
    assert_eq!(config.network.namespace, b"cli-net");
    assert_eq!(config.consensus.namespace, b"toml-cons");
    assert_eq!(config.network.max_message_size, 8192);
    assert_eq!(config.consensus.block_interval, Duration::from_millis(2500));
}

#[test]
fn tst_11_empty_bootstrap_validators_rejection() {
    let path = write_config_file("bootstrap_validators = []\n");
    let err = load_config(NodeArgs {
        config: Some(path),
        listen_addr: None,
        dialable_addr: None,
        bootstrap_peer: vec![],
        dial_peer: vec![],
        validator_seed: None,
        validator: vec![],
        rpc_addr: None,
        mem_rpc_addr: None,
        data_dir: None,
        max_message_size: None,
        network_namespace: None,
        consensus_namespace: None,
        block_interval_ms: None,
        genesis_bootstrap_dkg: false,
        genesis_bootstrap_validator_count: None,
        genesis_dkg_session_dir: None,
        genesis_dkg_dealer_pubkey: None,
    })
    .expect_err("empty validators should fail");

    assert!(matches!(err, ConfigError::EmptyBootstrapValidators));
}

#[test]
fn tst_12_legacy_validators_alias_maps_to_bootstrap_validators() {
    let validator = validator_hexes(&[13]).pop().unwrap();
    let path = write_config_file(&format!("validators = [\"{validator}\"]\n"));

    let config = load_config(NodeArgs {
        config: Some(path),
        listen_addr: None,
        dialable_addr: None,
        bootstrap_peer: vec![],
        dial_peer: vec![],
        validator_seed: None,
        validator: vec![],
        rpc_addr: None,
        mem_rpc_addr: None,
        data_dir: None,
        max_message_size: None,
        network_namespace: None,
        consensus_namespace: None,
        block_interval_ms: None,
        genesis_bootstrap_dkg: false,
        genesis_bootstrap_validator_count: None,
        genesis_dkg_session_dir: None,
        genesis_dkg_dealer_pubkey: None,
    })
    .expect("legacy validators alias should parse");

    assert_eq!(
        config
            .bootstrap_validators
            .expect("parsed validators")
            .len(),
        1
    );
}

#[test]
fn tst_13_genesis_bootstrap_flags_roundtrip() {
    let args = NodeArgs::parse_from([
        "whirlpool-node",
        "--genesis-bootstrap-dkg",
        "--genesis-bootstrap-validator-count",
        "4",
        "--genesis-dkg-session-dir",
        "/tmp/bootstrap-session",
    ]);

    let config = NodeConfig::from(args);
    assert!(config.bootstrap.genesis_bootstrap_dkg);
    assert_eq!(config.bootstrap.genesis_bootstrap_validator_count, Some(4));
    assert_eq!(
        config.bootstrap.genesis_dkg_session_dir,
        Some(PathBuf::from("/tmp/bootstrap-session"))
    );
    assert_eq!(config.bootstrap.genesis_dkg_dealer_pubkey, None);
}

#[test]
fn tst_14_genesis_bootstrap_rejects_zero_count() {
    let err = load_config(NodeArgs {
        config: None,
        listen_addr: None,
        dialable_addr: None,
        bootstrap_peer: vec![],
        dial_peer: vec![],
        validator_seed: None,
        validator: vec![],
        rpc_addr: None,
        mem_rpc_addr: None,
        data_dir: None,
        max_message_size: None,
        network_namespace: None,
        consensus_namespace: None,
        block_interval_ms: None,
        genesis_bootstrap_dkg: true,
        genesis_bootstrap_validator_count: Some(0),
        genesis_dkg_session_dir: None,
        genesis_dkg_dealer_pubkey: None,
    })
    .expect_err("zero bootstrap validator count must fail");

    assert!(matches!(
        err,
        ConfigError::InvalidGenesisBootstrapValidatorCount(0)
    ));
}

#[test]
fn tst_15_genesis_dkg_dealer_pubkey_roundtrip_and_invalid_parse() {
    let dealer = ed25519::PrivateKey::from_seed(77).public_key();
    let dealer_hex = hex(dealer.as_ref());
    let args = NodeArgs::parse_from(["whirlpool-node", "--genesis-dkg-dealer-pubkey", &dealer_hex]);
    let config = NodeConfig::from(args);
    assert_eq!(config.bootstrap.genesis_dkg_dealer_pubkey, Some(dealer));

    let err = load_config(NodeArgs {
        config: None,
        listen_addr: None,
        dialable_addr: None,
        bootstrap_peer: vec![],
        dial_peer: vec![],
        validator_seed: None,
        validator: vec![],
        rpc_addr: None,
        mem_rpc_addr: None,
        data_dir: None,
        max_message_size: None,
        network_namespace: None,
        consensus_namespace: None,
        block_interval_ms: None,
        genesis_bootstrap_dkg: false,
        genesis_bootstrap_validator_count: None,
        genesis_dkg_session_dir: None,
        genesis_dkg_dealer_pubkey: Some("not-hex".to_string()),
    })
    .expect_err("invalid dealer key must fail");
    assert!(matches!(
        err,
        ConfigError::InvalidGenesisDkgDealerPubkey { .. }
    ));
}
