pub mod local;

use crate::universal_path::UniversalPath;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{ops::Range, time::SystemTime};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageBackend {
    Local,
    Ftp,
    Sftp,
    S3,
}

impl StorageBackend {
    pub(crate) fn from_scheme(scheme: &str) -> Option<Self> {
        match scheme.to_lowercase().as_str() {
            "file" | "" => Some(StorageBackend::Local),
            "ftp" => Some(StorageBackend::Ftp),
            "sftp" => Some(StorageBackend::Sftp),
            "s3" => Some(StorageBackend::S3),
            _ => None,
        }
    }

    pub(crate) fn to_scheme(&self) -> &str {
        match self {
            StorageBackend::Local => "file",
            StorageBackend::Ftp => "ftp",
            StorageBackend::Sftp => "sftp",
            StorageBackend::S3 => "s3",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
    Other,
}

#[derive(Debug, Clone)]
pub struct EntryMetadata {
    pub kind: EntryKind,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<SystemTime>,
    pub created_at: Option<SystemTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageCapabilities {
    pub can_stat: bool,
    pub can_read: bool,
    pub can_read_range: bool,
    pub can_list: bool,
    pub can_glob: bool,
}

impl StorageCapabilities {
    pub const fn none() -> Self {
        StorageCapabilities {
            can_stat: false,
            can_read: false,
            can_read_range: false,
            can_list: false,
            can_glob: false,
        }
    }
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("unsupported backend: {0:?}")]
    UnsupportedBackend(StorageBackend),
    #[error("unsupported feature: {0}")]
    UnsupportedFeature(&'static str),
    #[error("invalid path for backend")]
    InvalidPath,
    #[error("not found")]
    NotFound,
    #[error("not a file")]
    NotAFile,
    #[error("not a directory")]
    NotADirectory,
    #[error("range not satisfiable")]
    RangeNotSatisfiable,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[async_trait]
pub trait Storage: Send + Sync {
    fn backend(&self) -> StorageBackend;
    fn capabilities(&self) -> StorageCapabilities;

    async fn stat(&self, path: &UniversalPath) -> Result<EntryMetadata, StorageError>;
    async fn read(&self, path: &UniversalPath) -> Result<Vec<u8>, StorageError>;
    async fn read_range(&self, path: &UniversalPath, range: Range<u64>)
        -> Result<Vec<u8>, StorageError>;
    async fn list(&self, path: &UniversalPath) -> Result<Vec<UniversalPath>, StorageError>;

    // Optional features with default implementations
    async fn glob(&self, _pattern: &UniversalPath) -> Result<Vec<UniversalPath>, StorageError> {
        Err(StorageError::UnsupportedFeature("glob"))
    }
}

/// Factory that returns a storage implementation for the given path's backend.
pub fn open_storage_for(path: &UniversalPath) -> Result<Box<dyn Storage>, StorageError> {
    match path.backend() {
        StorageBackend::Local => Ok(Box::new(local::LocalStorage::default())),
        other => Err(StorageError::UnsupportedBackend(other.clone())),
    }
}

pub use local::LocalStorage;
