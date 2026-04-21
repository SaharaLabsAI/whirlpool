use super::*;
use commonware_cryptography::Signer;
use commonware_cryptography::ed25519::PrivateKey;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(1);

fn temp_dir(label: &str) -> PathBuf {
    let id = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("consensus-manager-{label}-{id}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

fn participants(count: u64) -> Vec<ed25519::PublicKey> {
    (0..count)
        .map(|seed| PrivateKey::from_seed(seed + 1).public_key())
        .collect()
}

#[test]
fn bootstrap_writes_manifest_and_bundles() {
    let root = temp_dir("bootstrap");
    let participants = participants(4);

    let result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
        session_id: 7,
        output_dir: root.clone(),
        participants: participants.clone(),
    })
    .expect("bootstrap should succeed");

    assert!(result.manifest_path.exists());
    assert!(result.manifest_hash_path.exists());
    assert!(result.manifest_signature_path.exists());
    assert!(result.manifest_dealer_path.exists());
    assert_eq!(result.bundle_paths.len(), 4);
    for (player, path) in result.bundle_paths {
        assert!(participants.contains(&player));
        assert!(path.exists());
    }
}

#[test]
fn load_local_bundle_validates_share_material() {
    let root = temp_dir("load");
    let participants = participants(3);
    let local = participants[1].clone();

    let result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
        session_id: 9,
        output_dir: root,
        participants: participants.clone(),
    })
    .expect("bootstrap should succeed");

    let material = load_local_bundle(LoadLocalBundleConfig {
        session_dir: result.session_dir,
        local_validator: local.clone(),
        expected_dealer: result.dealer_public_key,
    })
    .expect("local bundle should load");

    assert_eq!(material.session_id, 9);
    assert_eq!(material.participants, participants);
    let ordered = Set::from_iter_dedup(material.participants.clone());
    let index = ordered.index(&local).expect("participant index");
    assert_eq!(
        material.share.public::<MinSig>(),
        material
            .polynomial
            .partial_public(index)
            .expect("partial public")
    );
}

#[test]
fn load_local_bundle_rejects_wrong_recipient_bundle() {
    let root = temp_dir("recipient-mismatch");
    let participants = participants(2);
    let local = participants[0].clone();
    let other = participants[1].clone();

    let result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
        session_id: 11,
        output_dir: root,
        participants,
    })
    .expect("bootstrap should succeed");

    let local_path = result
        .session_dir
        .join(BUNDLES_DIR)
        .join(bundle_file_name(&local));
    let other_path = result
        .session_dir
        .join(BUNDLES_DIR)
        .join(bundle_file_name(&other));
    fs::copy(&other_path, &local_path).expect("overwrite local bundle");

    let err = match load_local_bundle(LoadLocalBundleConfig {
        session_dir: result.session_dir,
        local_validator: local,
        expected_dealer: result.dealer_public_key,
    }) {
        Ok(_) => panic!("mismatched bundle recipient must fail"),
        Err(err) => err,
    };

    assert!(matches!(err, Error::RecipientMismatch { .. }));
}

#[test]
fn load_local_bundle_rejects_incomplete_session() {
    let root = temp_dir("incomplete-session");
    let participants = participants(3);
    let local = participants[0].clone();

    let result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
        session_id: 12,
        output_dir: root,
        participants: participants.clone(),
    })
    .expect("bootstrap should succeed");

    let missing_player = participants[2].clone();
    let missing_bundle_path = result
        .session_dir
        .join(BUNDLES_DIR)
        .join(bundle_file_name(&missing_player));
    fs::remove_file(&missing_bundle_path).expect("remove bundle to simulate incomplete session");

    let err = match load_local_bundle(LoadLocalBundleConfig {
        session_dir: result.session_dir,
        local_validator: local,
        expected_dealer: result.dealer_public_key,
    }) {
        Ok(_) => panic!("incomplete session must fail"),
        Err(err) => err,
    };

    match err {
        Error::MissingBundle(path) => assert_eq!(path, missing_bundle_path),
        other => panic!("expected missing bundle error, got {other}"),
    }
}

#[test]
fn load_local_bundle_rejects_session_mismatch_bundle() {
    let root = temp_dir("session-mismatch");
    let participants = participants(2);
    let local = participants[0].clone();

    let result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
        session_id: 15,
        output_dir: root,
        participants,
    })
    .expect("bootstrap should succeed");

    let local_bundle_path = result
        .session_dir
        .join(BUNDLES_DIR)
        .join(bundle_file_name(&local));
    let mut local_bundle = read_bundle(&local_bundle_path).expect("read local bundle");
    local_bundle.session_id = 99;
    fs::write(&local_bundle_path, local_bundle.encode()).expect("rewrite tampered local bundle");

    let err = match load_local_bundle(LoadLocalBundleConfig {
        session_dir: result.session_dir,
        local_validator: local,
        expected_dealer: result.dealer_public_key,
    }) {
        Ok(_) => panic!("session mismatch must fail"),
        Err(err) => err,
    };

    assert!(matches!(
        err,
        Error::SessionMismatch {
            expected: 15,
            found: 99
        }
    ));
}

#[test]
fn load_local_bundle_rejects_tampered_manifest_hash() {
    let root = temp_dir("manifest-hash");
    let participants = participants(2);
    let local = participants[0].clone();
    let result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
        session_id: 17,
        output_dir: root,
        participants,
    })
    .expect("bootstrap should succeed");

    let hash_path = result.session_dir.join(MANIFEST_HASH_FILE);
    fs::write(&hash_path, [0u8; SHA256_DIGEST_LEN]).expect("tamper manifest hash");

    let err = match load_local_bundle(LoadLocalBundleConfig {
        session_dir: result.session_dir,
        local_validator: local,
        expected_dealer: result.dealer_public_key,
    }) {
        Ok(_) => panic!("tampered hash must fail"),
        Err(err) => err,
    };
    assert!(matches!(err, Error::ManifestHashMismatch { .. }));
}

#[test]
fn load_local_bundle_rejects_wrong_dealer_key() {
    let root = temp_dir("dealer-mismatch");
    let participants = participants(2);
    let local = participants[0].clone();
    let result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
        session_id: 19,
        output_dir: root,
        participants,
    })
    .expect("bootstrap should succeed");
    let wrong_dealer = PrivateKey::from_seed(901).public_key();

    let err = match load_local_bundle(LoadLocalBundleConfig {
        session_dir: result.session_dir,
        local_validator: local,
        expected_dealer: wrong_dealer,
    }) {
        Ok(_) => panic!("wrong dealer key must fail"),
        Err(err) => err,
    };
    assert!(matches!(err, Error::DealerPublicKeyMismatch { .. }));
}

#[test]
fn load_local_bundle_rejects_tampered_manifest_signature() {
    let root = temp_dir("manifest-sig");
    let participants = participants(2);
    let local = participants[0].clone();
    let result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
        session_id: 20,
        output_dir: root,
        participants,
    })
    .expect("bootstrap should succeed");
    fs::write(&result.manifest_signature_path, [0u8; 64]).expect("tamper signature");

    let err = match load_local_bundle(LoadLocalBundleConfig {
        session_dir: result.session_dir,
        local_validator: local,
        expected_dealer: result.dealer_public_key,
    }) {
        Ok(_) => panic!("tampered signature must fail"),
        Err(err) => err,
    };
    assert!(matches!(err, Error::InvalidManifestSignature(..)));
}

#[test]
#[cfg(unix)]
fn bootstrap_artifacts_use_owner_only_permissions() {
    let root = temp_dir("permissions");
    let participants = participants(2);
    let result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
        session_id: 16,
        output_dir: root,
        participants,
    })
    .expect("bootstrap should succeed");

    let session_mode = fs::metadata(&result.session_dir)
        .expect("read session metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(session_mode, 0o700);

    let manifest_mode = fs::metadata(&result.manifest_path)
        .expect("read manifest metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(manifest_mode, 0o600);

    let hash_path = result.session_dir.join(MANIFEST_HASH_FILE);
    let hash_mode = fs::metadata(&hash_path)
        .expect("read hash metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(hash_mode, 0o600);

    let signature_mode = fs::metadata(&result.manifest_signature_path)
        .expect("read signature metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(signature_mode, 0o600);

    let dealer_mode = fs::metadata(&result.manifest_dealer_path)
        .expect("read dealer key metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(dealer_mode, 0o600);

    for (_, bundle_path) in result.bundle_paths {
        let bundle_mode = fs::metadata(&bundle_path)
            .expect("read bundle metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(bundle_mode, 0o600);
    }
}
