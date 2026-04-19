//! Trusted-dealer bootstrap manager for consensus key material.
#![allow(clippy::result_large_err)]

use bytes::{Buf, BufMut};
use commonware_codec::{Encode, EncodeSize, Error as CodecError, Read, ReadExt, Write};
use commonware_cryptography::{
    bls12381::{
        dkg,
        primitives::{group::Share, sharing::ModeVersion, variant::MinSig},
    },
    ed25519,
    sha256::Sha256,
    Hasher, Signer, Verifier,
};
use commonware_utils::{
    hex,
    ordered::{Map, Quorum, Set},
    N3f1,
};
use rand::rngs::OsRng;
use rand::RngCore;
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::{
    fmt, fs,
    fs::OpenOptions,
    num::NonZeroU32,
    path::{Path, PathBuf},
};

const MANIFEST_FILE: &str = "manifest.bin";
const MANIFEST_HASH_FILE: &str = "manifest.sha256";
const MANIFEST_SIGNATURE_FILE: &str = "manifest.sig";
const MANIFEST_DEALER_FILE: &str = "manifest.dealer";
const BUNDLES_DIR: &str = "bundles";
const BUNDLE_EXT: &str = "bundle";
const MAX_MANIFEST_PARTICIPANTS: u32 = 10_000;
const SHA256_DIGEST_LEN: usize = 32;
const MANIFEST_SIGNATURE_NAMESPACE: &[u8] = b"whirlpool-dkg-manifest-v1";

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Codec(CodecError),
    InvalidParticipants(&'static str),
    MissingBundle(PathBuf),
    MissingManifestHash(PathBuf),
    MissingManifestSignature(PathBuf),
    MissingManifestDealer(PathBuf),
    InvalidManifestHashLength {
        expected: usize,
        found: usize,
    },
    DealerPublicKeyMismatch {
        expected: ed25519::PublicKey,
        found: ed25519::PublicKey,
    },
    ManifestHashMismatch {
        expected: [u8; 32],
        found: [u8; 32],
    },
    InvalidManifestSignature(ed25519::PublicKey),
    MissingPlayer(ed25519::PublicKey),
    MissingShare(ed25519::PublicKey),
    SessionMismatch {
        expected: u64,
        found: u64,
    },
    RecipientMismatch {
        expected: ed25519::PublicKey,
        found: ed25519::PublicKey,
    },
    ShareVerificationFailed(ed25519::PublicKey),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Codec(err) => write!(f, "codec error: {err}"),
            Self::InvalidParticipants(msg) => write!(f, "invalid participants: {msg}"),
            Self::MissingBundle(path) => write!(f, "missing bundle file: {}", path.display()),
            Self::MissingManifestHash(path) => {
                write!(f, "missing manifest hash file: {}", path.display())
            }
            Self::MissingManifestSignature(path) => {
                write!(f, "missing manifest signature file: {}", path.display())
            }
            Self::MissingManifestDealer(path) => {
                write!(f, "missing manifest dealer file: {}", path.display())
            }
            Self::InvalidManifestHashLength { expected, found } => write!(
                f,
                "invalid manifest hash length: expected {expected} bytes, found {found}"
            ),
            Self::DealerPublicKeyMismatch { expected, found } => write!(
                f,
                "manifest dealer key mismatch: expected {}, found {}",
                hex(expected.as_ref()),
                hex(found.as_ref())
            ),
            Self::ManifestHashMismatch { expected, found } => write!(
                f,
                "manifest hash mismatch: expected {}, found {}",
                hex(expected),
                hex(found)
            ),
            Self::InvalidManifestSignature(dealer) => write!(
                f,
                "manifest signature verification failed for dealer {}",
                hex(dealer.as_ref())
            ),
            Self::MissingPlayer(player) => {
                write!(f, "player not found in manifest: {}", hex(player.as_ref()))
            }
            Self::MissingShare(player) => {
                write!(f, "missing share for player: {}", hex(player.as_ref()))
            }
            Self::SessionMismatch { expected, found } => {
                write!(f, "session id mismatch: expected {expected}, found {found}")
            }
            Self::RecipientMismatch { expected, found } => write!(
                f,
                "bundle recipient mismatch: expected {}, found {}",
                hex(expected.as_ref()),
                hex(found.as_ref())
            ),
            Self::ShareVerificationFailed(player) => write!(
                f,
                "share verification failed for player {}",
                hex(player.as_ref())
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Codec(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<CodecError> for Error {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}

#[derive(Clone, Debug)]
pub struct TrustedDealerBootstrapConfig {
    pub session_id: u64,
    pub output_dir: PathBuf,
    pub participants: Vec<ed25519::PublicKey>,
}

#[derive(Clone, Debug)]
pub struct TrustedDealerBootstrapResult {
    pub session_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest_hash_path: PathBuf,
    pub manifest_signature_path: PathBuf,
    pub manifest_dealer_path: PathBuf,
    pub dealer_public_key: ed25519::PublicKey,
    pub bundle_paths: Vec<(ed25519::PublicKey, PathBuf)>,
}

#[derive(Clone, Debug)]
pub struct LoadLocalBundleConfig {
    pub session_dir: PathBuf,
    pub local_validator: ed25519::PublicKey,
    pub expected_dealer: ed25519::PublicKey,
}

#[derive(Clone)]
pub struct LocalBundleMaterial {
    pub session_id: u64,
    pub dealers: Vec<ed25519::PublicKey>,
    pub participants: Vec<ed25519::PublicKey>,
    pub polynomial: commonware_cryptography::bls12381::primitives::sharing::Sharing<MinSig>,
    pub share: Share,
    pub manifest_path: PathBuf,
    pub bundle_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BootstrapManifest {
    session_id: u64,
    output: dkg::Output<MinSig, ed25519::PublicKey>,
}

impl EncodeSize for BootstrapManifest {
    fn encode_size(&self) -> usize {
        self.session_id.encode_size() + self.output.encode_size()
    }
}

impl Write for BootstrapManifest {
    fn write(&self, buf: &mut impl BufMut) {
        self.session_id.write(buf);
        self.output.write(buf);
    }
}

impl Read for BootstrapManifest {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _: &()) -> Result<Self, CodecError> {
        let session_id: u64 = ReadExt::read(buf)?;
        let output = dkg::Output::<MinSig, ed25519::PublicKey>::read_cfg(
            buf,
            &(
                NonZeroU32::new(MAX_MANIFEST_PARTICIPANTS)
                    .expect("manifest participant bound is non-zero"),
                ModeVersion::v0(),
            ),
        )?;
        Ok(Self { session_id, output })
    }
}

#[derive(Clone, PartialEq, Eq)]
struct ValidatorBundle {
    session_id: u64,
    recipient: ed25519::PublicKey,
    share: Share,
}

impl EncodeSize for ValidatorBundle {
    fn encode_size(&self) -> usize {
        self.session_id.encode_size() + self.recipient.encode_size() + self.share.encode_size()
    }
}

impl Write for ValidatorBundle {
    fn write(&self, buf: &mut impl BufMut) {
        self.session_id.write(buf);
        self.recipient.write(buf);
        self.share.write(buf);
    }
}

impl Read for ValidatorBundle {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _: &()) -> Result<Self, CodecError> {
        Ok(Self {
            session_id: ReadExt::read(buf)?,
            recipient: ReadExt::read(buf)?,
            share: Read::read_cfg(buf, &())?,
        })
    }
}

pub fn run_trusted_dealer_bootstrap(
    config: TrustedDealerBootstrapConfig,
) -> Result<TrustedDealerBootstrapResult, Error> {
    if config.participants.is_empty() {
        return Err(Error::InvalidParticipants(
            "must provide at least one participant",
        ));
    }

    let participant_set = Set::from_iter_dedup(config.participants.clone());
    if participant_set.len() != config.participants.len() {
        return Err(Error::InvalidParticipants(
            "participants must be unique and ordered deterministically",
        ));
    }

    let (output, shares) = dkg::deal::<MinSig, _, N3f1>(OsRng, Default::default(), participant_set)
        .map_err(|_| Error::InvalidParticipants("failed to generate trusted dealer shares"))?;

    let session_dir = config
        .output_dir
        .join(format!("session-{:016x}", config.session_id));
    let bundles_dir = session_dir.join(BUNDLES_DIR);
    fs::create_dir_all(&bundles_dir)?;
    harden_directory_permissions(&session_dir)?;
    harden_directory_permissions(&bundles_dir)?;

    let manifest = BootstrapManifest {
        session_id: config.session_id,
        output,
    };
    let manifest_path = session_dir.join(MANIFEST_FILE);
    let manifest_hash_path = session_dir.join(MANIFEST_HASH_FILE);
    let manifest_signature_path = session_dir.join(MANIFEST_SIGNATURE_FILE);
    let manifest_dealer_path = session_dir.join(MANIFEST_DEALER_FILE);
    let manifest_bytes = manifest.encode();
    let dealer_signer = generate_dealer_signer();
    let dealer_public_key = dealer_signer.public_key();
    let manifest_signature = dealer_signer.sign(MANIFEST_SIGNATURE_NAMESPACE, &manifest_bytes);
    write_private_file(&manifest_path, &manifest_bytes)?;
    write_private_file(
        &manifest_hash_path,
        manifest_digest(&manifest_bytes).as_ref(),
    )?;
    write_private_file(&manifest_signature_path, &manifest_signature.encode())?;
    write_private_file(&manifest_dealer_path, &dealer_public_key.encode())?;

    let mut bundle_paths = Vec::with_capacity(config.participants.len());
    write_bundles(&manifest, &shares, &bundles_dir, &mut bundle_paths)?;

    Ok(TrustedDealerBootstrapResult {
        session_dir,
        manifest_path,
        manifest_hash_path,
        manifest_signature_path,
        manifest_dealer_path,
        dealer_public_key,
        bundle_paths,
    })
}

pub fn load_local_bundle(config: LoadLocalBundleConfig) -> Result<LocalBundleMaterial, Error> {
    let manifest_path = config.session_dir.join(MANIFEST_FILE);
    validate_manifest_authenticity(&config.session_dir, &manifest_path, &config.expected_dealer)?;
    let manifest = read_manifest(&manifest_path)?;
    validate_session_is_complete(&config.session_dir, &manifest)?;

    let bundle_path = config
        .session_dir
        .join(BUNDLES_DIR)
        .join(bundle_file_name(&config.local_validator));
    if !bundle_path.exists() {
        return Err(Error::MissingBundle(bundle_path));
    }

    let bundle = read_bundle(&bundle_path)?;
    validate_bundle(&manifest, &bundle, config.local_validator.clone())?;

    Ok(LocalBundleMaterial {
        session_id: manifest.session_id,
        dealers: manifest.output.dealers().iter().cloned().collect(),
        participants: manifest.output.players().iter().cloned().collect(),
        polynomial: manifest.output.public().clone(),
        share: bundle.share,
        manifest_path,
        bundle_path,
    })
}

fn validate_session_is_complete(
    session_dir: &Path,
    manifest: &BootstrapManifest,
) -> Result<(), Error> {
    let bundles_dir = session_dir.join(BUNDLES_DIR);
    for player in manifest.output.players().iter() {
        let bundle_path = bundles_dir.join(bundle_file_name(player));
        if !bundle_path.exists() {
            return Err(Error::MissingBundle(bundle_path));
        }
        let bundle = read_bundle(&bundle_path)?;
        validate_bundle(manifest, &bundle, player.clone())?;
    }
    Ok(())
}

fn read_manifest(path: &Path) -> Result<BootstrapManifest, Error> {
    let bytes = fs::read(path)?;
    decode_exact::<BootstrapManifest>(bytes)
}

fn read_bundle(path: &Path) -> Result<ValidatorBundle, Error> {
    let bytes = fs::read(path)?;
    decode_exact::<ValidatorBundle>(bytes)
}

fn validate_manifest_authenticity(
    session_dir: &Path,
    manifest_path: &Path,
    expected_dealer: &ed25519::PublicKey,
) -> Result<(), Error> {
    let manifest_hash_path = session_dir.join(MANIFEST_HASH_FILE);
    if !manifest_hash_path.exists() {
        return Err(Error::MissingManifestHash(manifest_hash_path));
    }
    let manifest_signature_path = session_dir.join(MANIFEST_SIGNATURE_FILE);
    if !manifest_signature_path.exists() {
        return Err(Error::MissingManifestSignature(manifest_signature_path));
    }
    let manifest_dealer_path = session_dir.join(MANIFEST_DEALER_FILE);
    if !manifest_dealer_path.exists() {
        return Err(Error::MissingManifestDealer(manifest_dealer_path));
    }

    let manifest_bytes = fs::read(manifest_path)?;
    let found_hash = fs::read(&manifest_hash_path)?;
    if found_hash.len() != SHA256_DIGEST_LEN {
        return Err(Error::InvalidManifestHashLength {
            expected: SHA256_DIGEST_LEN,
            found: found_hash.len(),
        });
    }

    let expected_hash = manifest_digest(&manifest_bytes);
    let mut found = [0u8; SHA256_DIGEST_LEN];
    found.copy_from_slice(&found_hash);
    if expected_hash != found {
        return Err(Error::ManifestHashMismatch {
            expected: expected_hash,
            found,
        });
    }

    let found_dealer = decode_exact::<ed25519::PublicKey>(fs::read(&manifest_dealer_path)?)?;
    if &found_dealer != expected_dealer {
        return Err(Error::DealerPublicKeyMismatch {
            expected: expected_dealer.clone(),
            found: found_dealer,
        });
    }

    let signature = decode_exact::<ed25519::Signature>(fs::read(&manifest_signature_path)?)?;
    if !expected_dealer.verify(MANIFEST_SIGNATURE_NAMESPACE, &manifest_bytes, &signature) {
        return Err(Error::InvalidManifestSignature(expected_dealer.clone()));
    }

    Ok(())
}

fn manifest_digest(bytes: &[u8]) -> [u8; SHA256_DIGEST_LEN] {
    let digest = Sha256::hash(bytes);
    let mut out = [0u8; SHA256_DIGEST_LEN];
    out.copy_from_slice(digest.as_ref());
    out
}

fn generate_dealer_signer() -> ed25519::PrivateKey {
    let mut raw_seed = [0u8; 32];
    OsRng.fill_bytes(&mut raw_seed);
    let mut reader = raw_seed.as_slice();
    // any 32-byte value is accepted by ed25519_consensus::SigningKey
    ed25519::PrivateKey::read_cfg(&mut reader, &())
        .expect("os rng produced invalid dealer private key bytes")
}

fn decode_exact<T: Read<Cfg = ()>>(bytes: Vec<u8>) -> Result<T, Error> {
    let mut reader = bytes.as_slice();
    let decoded = T::read_cfg(&mut reader, &())?;
    if !reader.is_empty() {
        return Err(Error::Codec(CodecError::Invalid(
            "bootstrap artifact",
            "trailing bytes",
        )));
    }
    Ok(decoded)
}

fn write_bundles(
    manifest: &BootstrapManifest,
    shares: &Map<ed25519::PublicKey, Share>,
    bundles_dir: &Path,
    out_paths: &mut Vec<(ed25519::PublicKey, PathBuf)>,
) -> Result<(), Error> {
    for player in manifest.output.players().iter() {
        let Some(share) = shares.get_value(player).cloned() else {
            return Err(Error::MissingShare(player.clone()));
        };

        let bundle = ValidatorBundle {
            session_id: manifest.session_id,
            recipient: player.clone(),
            share,
        };
        let bundle_path = bundles_dir.join(bundle_file_name(player));
        write_private_file(&bundle_path, &bundle.encode())?;
        out_paths.push((player.clone(), bundle_path));
    }
    Ok(())
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let mut file = OpenOptions::new();
    file.write(true).create_new(true);
    #[cfg(unix)]
    {
        file.mode(0o600);
    }
    let mut file = file.open(path)?;
    use std::io::Write as _;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(())
}

fn harden_directory_permissions(path: &Path) -> Result<(), Error> {
    #[cfg(unix)]
    {
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn validate_bundle(
    manifest: &BootstrapManifest,
    bundle: &ValidatorBundle,
    expected_recipient: ed25519::PublicKey,
) -> Result<(), Error> {
    if bundle.session_id != manifest.session_id {
        return Err(Error::SessionMismatch {
            expected: manifest.session_id,
            found: bundle.session_id,
        });
    }

    if bundle.recipient != expected_recipient {
        return Err(Error::RecipientMismatch {
            expected: expected_recipient,
            found: bundle.recipient.clone(),
        });
    }

    let Some(index) = manifest.output.players().index(&bundle.recipient) else {
        return Err(Error::MissingPlayer(bundle.recipient.clone()));
    };

    let expected_public = manifest
        .output
        .public()
        .partial_public(index)
        .map_err(|_| Error::MissingPlayer(bundle.recipient.clone()))?;

    if expected_public != bundle.share.public::<MinSig>() {
        return Err(Error::ShareVerificationFailed(bundle.recipient.clone()));
    }

    Ok(())
}

fn bundle_file_name(player: &ed25519::PublicKey) -> String {
    format!("{}.{}", hex(player.as_ref()), BUNDLE_EXT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_cryptography::ed25519::PrivateKey;
    use commonware_cryptography::Signer;
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
        fs::remove_file(&missing_bundle_path)
            .expect("remove bundle to simulate incomplete session");

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
        fs::write(&local_bundle_path, local_bundle.encode())
            .expect("rewrite tampered local bundle");

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
}
