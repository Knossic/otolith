mod backends;
mod universal_path;

pub use backends::{
    open_storage_for, EntryKind, EntryMetadata, Storage, StorageBackend, StorageCapabilities,
    StorageError,
};
pub use universal_path::{UniversalPath, UniversalPathError};

 
