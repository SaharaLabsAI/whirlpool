use commonware_codec::Error as CodecError;
use commonware_cryptography::{
    bls12381::primitives::{group::Share, variant::MinSig},
    ed25519,
};
use commonware_utils::hex;
use std::{fmt, path::PathBuf};

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
