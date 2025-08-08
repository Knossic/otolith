use super::{EntryKind, EntryMetadata, Storage, StorageBackend, StorageCapabilities, StorageError};
use crate::universal_path::UniversalPath;
use async_trait::async_trait;
use std::path::PathBuf;

#[derive(Default)]
pub struct LocalStorage;

impl LocalStorage {
    fn to_pathbuf(&self, upath: &UniversalPath) -> Result<PathBuf, StorageError> {
        if upath.backend() != &StorageBackend::Local {
            return Err(StorageError::InvalidPath);
        }

        let segments = upath.path_segments();
        #[cfg(windows)]
        {
            use std::path::Path;
            if segments.first().map(|s| s.ends_with(':')).unwrap_or(false) {
                let mut pb = PathBuf::from(segments[0].clone());
                for seg in &segments[1..] {
                    pb.push(seg);
                }
                return Ok(pb);
            }
            let mut pb = PathBuf::new();
            pb.push(Path::new("/"));
            for seg in segments {
                pb.push(seg);
            }
            Ok(pb)
        }
        #[cfg(not(windows))]
        {
            let mut pb = PathBuf::from("/");
            for seg in segments {
                pb.push(seg);
            }
            Ok(pb)
        }
    }
}

#[async_trait]
impl Storage for LocalStorage {
    fn backend(&self) -> StorageBackend {
        StorageBackend::Local
    }

    fn capabilities(&self) -> StorageCapabilities {
        StorageCapabilities {
            can_stat: true,
            can_read: true,
            can_read_range: true,
            can_list: true,
            can_glob: false,
        }
    }

    async fn stat(&self, path: &UniversalPath) -> Result<EntryMetadata, StorageError> {
        use tokio::fs;
        let pb = self.to_pathbuf(path)?;
        let md = fs::metadata(pb).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => StorageError::NotFound,
            _ => StorageError::Io(e),
        })?;

        let kind = if md.is_dir() {
            EntryKind::Directory
        } else if md.is_file() {
            EntryKind::File
        } else {
            EntryKind::Other
        };

        let size_bytes = if md.is_file() { Some(md.len()) } else { None };
        let modified_at = md.modified().ok();
        let created_at = md.created().ok();

        Ok(EntryMetadata {
            kind,
            size_bytes,
            modified_at,
            created_at,
        })
    }

    async fn read(&self, path: &UniversalPath) -> Result<Vec<u8>, StorageError> {
        use tokio::{fs::File, io::AsyncReadExt};
        let pb = self.to_pathbuf(path)?;
        let mut file = File::open(pb).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => StorageError::NotFound,
            _ => StorageError::Io(e),
        })?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;
        Ok(buf)
    }

    async fn read_range(
        &self,
        path: &UniversalPath,
        range: std::ops::Range<u64>,
    ) -> Result<Vec<u8>, StorageError> {
        use tokio::{
            fs::File,
            io::{AsyncReadExt, AsyncSeekExt},
        };

        if range.start >= range.end {
            return Ok(Vec::new());
        }

        let pb = self.to_pathbuf(path)?;
        let mut file = File::open(pb).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => StorageError::NotFound,
            _ => StorageError::Io(e),
        })?;

        let md = file.metadata().await?;
        if !md.is_file() {
            return Err(StorageError::NotAFile);
        }
        let len = md.len();
        if range.start >= len {
            return Err(StorageError::RangeNotSatisfiable);
        }

        let end = range.end.min(len);
        let to_read = (end - range.start) as usize;

        file.seek(std::io::SeekFrom::Start(range.start)).await?;
        let mut buf = vec![0u8; to_read];
        let mut read_so_far = 0usize;
        while read_so_far < to_read {
            let n = file.read(&mut buf[read_so_far..]).await?;
            if n == 0 {
                break;
            }
            read_so_far += n;
        }
        buf.truncate(read_so_far);
        Ok(buf)
    }

    async fn list(&self, path: &UniversalPath) -> Result<Vec<UniversalPath>, StorageError> {
        use tokio::fs;
        let pb = self.to_pathbuf(path)?;
        let md = fs::metadata(&pb).await?;
        if !md.is_dir() {
            return Err(StorageError::NotADirectory);
        }

        let mut entries = Vec::new();
        let mut rd = fs::read_dir(&pb).await?;
        while let Some(entry) = rd.next_entry().await? {
            let child_pb = entry.path();
            let display = child_pb.to_string_lossy().to_string();
            entries.push(UniversalPath::local(display));
        }
        Ok(entries)
    }
}


