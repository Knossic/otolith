use fluent_uri::{Uri, encoding::{EStr}, component::Scheme};
use std::fmt;
use std::str::FromStr;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageBackend {
    Local,
    NetworkDrive,
    Ftp,
    Sftp,
    S3,
    Http,
    Https,
    Other(String),
}

impl StorageBackend {
    fn from_scheme(scheme: &str) -> Self {
        match scheme.to_lowercase().as_str() {
            "file" | "" => StorageBackend::Local,
            "smb" | "cifs" => StorageBackend::NetworkDrive,
            "ftp" => StorageBackend::Ftp,
            "sftp" => StorageBackend::Sftp,
            "s3" => StorageBackend::S3,
            "http" => StorageBackend::Http,
            "https" => StorageBackend::Https,
            other => StorageBackend::Other(other.to_string()),
        }
    }

    fn to_scheme(&self) -> &str {
        match self {
            StorageBackend::Local => "file",
            StorageBackend::NetworkDrive => "smb",
            StorageBackend::Ftp => "ftp",
            StorageBackend::Sftp => "sftp",
            StorageBackend::S3 => "s3",
            StorageBackend::Http => "http",
            StorageBackend::Https => "https",
            StorageBackend::Other(scheme) => scheme,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UniversalPath {
    backend: StorageBackend,
    host: Option<String>,
    port: Option<u16>,
    path_segments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UniversalPathError {
    InvalidUri(String),
    EmptyPath,
    InvalidOperation(String),
}

impl fmt::Display for UniversalPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UniversalPathError::InvalidUri(msg) => write!(f, "Invalid URI: {}", msg),
            UniversalPathError::EmptyPath => write!(f, "Path is empty"),
            UniversalPathError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
        }
    }
}

impl std::error::Error for UniversalPathError {}

impl UniversalPath {
    /// Create a new UniversalPath from a URI string
    pub fn from_uri_str(uri_str: &str) -> Result<Self, UniversalPathError> {
        let uri = Uri::parse(uri_str)
            .map_err(|e| UniversalPathError::InvalidUri(format!("Failed to parse URI: {}", e)))?;
        Self::from_uri(uri)
    }

    /// Create a new UniversalPath from a fluent_uri::Uri
    pub fn from_uri(uri: Uri<&str>) -> Result<Self, UniversalPathError> {
        let scheme = uri.scheme().as_str();
        let backend = StorageBackend::from_scheme(scheme);
        
        let host = uri.authority()
            .map(|auth| auth.host().to_string());
        
        let port = uri.authority()
            .and_then(|auth| auth.port_to_u16().ok())
            .flatten();

        let path = uri.path().as_str();
        let path_segments = Self::split_path(path);

        Ok(UniversalPath {
            backend,
            host,
            port,
            path_segments,
        })
    }

    /// Create a new UniversalPath for local filesystem
    pub fn local<P: AsRef<str>>(path: P) -> Self {
        let path_segments = Self::split_path_local(path.as_ref());
        UniversalPath {
            backend: StorageBackend::Local,
            host: None,
            port: None,
            path_segments,
        }
    }

    /// Split a path string into segments for local filesystem (handles both POSIX and Windows)
    fn split_path_local(path: &str) -> Vec<String> {
        if path.is_empty() {
            return vec![];
        }

        let mut segments = Vec::new();
        
        // Handle Windows drive letters (e.g., "C:", "C:\", etc.)
        if path.len() >= 2 && path.chars().nth(1) == Some(':') {
            if let Some(drive_end) = path.find(':') {
                if drive_end == 1 {
                    // This looks like a drive letter
                    let drive = &path[..=drive_end]; // Include the colon
                    segments.push(drive.to_string());
                    
                    // Process the rest of the path after the drive
                    let remaining = &path[drive_end + 1..];
                    if !remaining.is_empty() {
                        // Skip leading separator if present
                        let remaining = remaining.strip_prefix('\\').or_else(|| remaining.strip_prefix('/')).unwrap_or(remaining);
                        if !remaining.is_empty() {
                            segments.extend(Self::split_path_segments(remaining));
                        }
                    }
                    return segments;
                }
            }
        }
        
        // Handle UNC paths on Windows (\\server\share)
        if path.starts_with("\\\\") || path.starts_with("//") {
            let remaining = &path[2..]; // Skip the leading //
            segments.extend(Self::split_path_segments(remaining));
            return segments;
        }
        
        // Handle regular paths (POSIX or Windows without drive letters)
        segments.extend(Self::split_path_segments(path));
        segments
    }

    /// Split a path string into segments (for URI paths)
    fn split_path(path: &str) -> Vec<String> {
        if path.is_empty() || path == "/" {
            return vec![];
        }

        Self::split_path_segments(path)
    }
    
    /// Helper function to split path segments using both / and \ as separators
    fn split_path_segments(path: &str) -> Vec<String> {
        // Skip leading separators
        let path = path.strip_prefix('/').or_else(|| path.strip_prefix('\\')).unwrap_or(path);
        
        // Split on both / and \ to handle mixed separators
        path.split(|c| c == '/' || c == '\\')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Join path segments back into a string
    fn join_path(segments: &[String]) -> String {
        if segments.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", segments.join("/"))
        }
    }

    /// Get the storage backend type
    pub fn backend(&self) -> &StorageBackend {
        &self.backend
    }

    /// Get the host (if applicable)
    pub fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    /// Get the port (if applicable)
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Get all path segments
    pub fn path_segments(&self) -> &[String] {
        &self.path_segments
    }

    /// Check if this path represents the root
    pub fn is_root(&self) -> bool {
        self.path_segments.is_empty()
    }

    /// Get the full path as a string
    pub fn path(&self) -> String {
        Self::join_path(&self.path_segments)
    }

    /// Get the last segment of the path
    pub fn last_segment(&self) -> Option<&str> {
        self.path_segments.last().map(|s| s.as_str())
    }

    /// Get the file extension
    pub fn extension(&self) -> Option<&str> {
        self.last_segment()?
            .rfind('.')
            .map(|pos| &self.last_segment().unwrap()[pos + 1..])
    }

    /// Append a segment to the path
    pub fn append<S: AsRef<str>>(&mut self, segment: S) -> &mut Self {
        let segment = segment.as_ref();
        if !segment.is_empty() && segment != "." {
            if segment == ".." {
                self.path_segments.pop();
            } else {
                self.path_segments.push(segment.to_string());
            }
        }
        self
    }

    /// Create a new path with an appended segment
    pub fn join<S: AsRef<str>>(&self, segment: S) -> UniversalPath {
        let mut new_path = self.clone();
        new_path.append(segment);
        new_path
    }

    /// Pop the last segment from the path
    pub fn pop(&mut self) -> Option<String> {
        self.path_segments.pop()
    }

    /// Get the parent directory
    pub fn parent(&self) -> Option<UniversalPath> {
        if self.is_root() {
            None
        } else {
            let mut dir_segments = self.path_segments.clone();
            dir_segments.pop();
            
            Some(UniversalPath {
                backend: self.backend.clone(),
                host: self.host.clone(),
                port: self.port,
                path_segments: dir_segments,
            })
        }
    }

    /// Convert to a URI string
    pub fn to_uri(&self) -> Result<String, UniversalPathError> {
        let scheme = Scheme::new(self.backend.to_scheme());
        if scheme.is_none() {
            return Err(UniversalPathError::InvalidUri(String::from("Invalid scheme in to_uri()")));
        }

        let uri = Uri::builder().scheme(scheme.expect("Scheme should not be none"));

        let uri = if let Some(host) = self.host.as_deref() {
            let estr_host = EStr::new(host);
            if estr_host.is_none() {
                return Err(UniversalPathError::InvalidUri(String::from("Invalid host in to_uri()")));
            }
            
            uri.authority_with(|b| {
                let builder = b.host(estr_host.unwrap());
                if let Some(port) = self.port {
                    builder.port(port)
                } else {
                    builder.advance()
                }
            })
        } else {
            // No host, advance to next state
            uri.advance()
        };

        // Encode the path for URI safety
        let path = self.path();
        let encoded_path = EStr::new(path.as_str());
        if encoded_path.is_none() {
            return Err(UniversalPathError::InvalidUri(String::from("Invalid path in to_uri()")));
        }

        return uri.path(encoded_path.unwrap())
            .build()
            .map(|t| t.into_string())
            .map_err(
                |e| UniversalPathError::InvalidUri(String::from(format!("Failed to convert to URI string in to_uri(): {}", e))));
    }

    /// Check if this path is a child of the given parent path.
    /// Returns Some(relative_segments) if this path is a child, None otherwise.
    /// The relative_segments contain the path segments relative to the parent.
    pub fn relative_to(&self, parent: &UniversalPath) -> Option<Vec<String>> {
        // Must have same backend
        if self.backend != parent.backend {
            return None;
        }
        
        // Must have same host
        if self.host != parent.host {
            return None;
        }
        
        // Must have same port
        if self.port != parent.port {
            return None;
        }
        
        // Check if parent's path segments are a prefix of this path's segments
        if self.path_segments.len() < parent.path_segments.len() {
            return None;
        }
        
        for (i, parent_segment) in parent.path_segments.iter().enumerate() {
            if self.path_segments.get(i) != Some(parent_segment) {
                return None;
            }
        }
        
        // Return the remaining segments
        Some(self.path_segments[parent.path_segments.len()..].to_vec())
    }
}

impl fmt::Display for UniversalPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_uri() {
            Ok(uri) => write!(f, "{}", uri),
            Err(_) => write!(f, "{}:{}", self.backend.to_scheme(), self.path()),
        }
    }
}

impl FromStr for UniversalPath {
    type Err = UniversalPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_uri_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_filesystem() {
        let local_path = UniversalPath::local("/home/user/music/song.mp3");
        
        assert_eq!(local_path.backend(), &StorageBackend::Local);
        assert_eq!(local_path.last_segment(), Some("song.mp3"));
        assert_eq!(local_path.extension(), Some("mp3"));
        assert!(!local_path.is_root());
        
        let parent = local_path.parent().unwrap();
        assert_eq!(parent.path_segments(), &["home", "user", "music"]);
    }

    #[test]
    fn test_s3_bucket() {
        let s3_path = UniversalPath::from_uri_str("s3://my-bucket/music/album/track.flac").unwrap();
        
        assert_eq!(s3_path.backend(), &StorageBackend::S3);
        assert_eq!(s3_path.host(), Some("my-bucket"));
        assert_eq!(s3_path.last_segment(), Some("track.flac"));
        assert_eq!(s3_path.extension(), Some("flac"));
    }

    #[test]
    fn test_sftp() {
        let sftp_path = UniversalPath::from_uri_str("sftp://music.server.com:22/media/collection/jazz/file.wav").unwrap();
        
        assert_eq!(sftp_path.backend(), &StorageBackend::Sftp);
        assert_eq!(sftp_path.host(), Some("music.server.com"));
        assert_eq!(sftp_path.port(), Some(22));
        assert_eq!(sftp_path.path_segments(), &["media", "collection", "jazz", "file.wav"]);
    }

    #[test]
    fn test_path_manipulation() {
        let mut path = UniversalPath::local("/music");
        assert_eq!(path.path(), "/music");
        
        path.append("classical");
        assert_eq!(path.path(), "/music/classical");
        
        path.append("beethoven");
        assert_eq!(path.path(), "/music/classical/beethoven");
        
        path.append("symphony_9.mp3");
        assert_eq!(path.path(), "/music/classical/beethoven/symphony_9.mp3");
        
        let parent = path.parent().unwrap();
        assert_eq!(parent.path(), "/music/classical/beethoven");
        
        let popped = path.pop();
        assert_eq!(popped, Some("symphony_9.mp3".to_string()));
        assert_eq!(path.path(), "/music/classical/beethoven");
        
        // Join operation (immutable)
        let new_path = path.join("concerto_1.flac");
        assert_eq!(new_path.path(), "/music/classical/beethoven/concerto_1.flac");
        assert_eq!(path.path(), "/music/classical/beethoven"); // Original unchanged
    }

    #[test]
    fn test_network_drive() {
        let network_path = UniversalPath::from_uri_str("smb://nas.local/music/library").unwrap();
        
        assert_eq!(network_path.backend(), &StorageBackend::NetworkDrive);
        assert_eq!(network_path.host(), Some("nas.local"));
        assert_eq!(network_path.path_segments(), &["music", "library"]);
    }

    #[test]
    fn test_error_handling() {
        // Invalid URI
        let result = UniversalPath::from_uri_str("://invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_root_checking() {
        let root_path = UniversalPath::local("/");
        assert!(root_path.is_root());
        assert_eq!(root_path.path_segments().len(), 0);
        
        let non_root = UniversalPath::local("/music");
        assert!(!non_root.is_root());
        assert_eq!(non_root.path_segments(), &["music"]);
    }

    #[test]
    fn test_windows_paths() {
        // Test Windows drive letters
        let drive_path = UniversalPath::local("C:\\Users\\Music\\song.mp3");
        assert_eq!(drive_path.path_segments(), &["C:", "Users", "Music", "song.mp3"]);
        assert_eq!(drive_path.last_segment(), Some("song.mp3"));
        
        // Test Windows drive without backslash
        let drive_only = UniversalPath::local("D:");
        assert_eq!(drive_only.path_segments(), &["D:"]);
        
        // Test mixed separators
        let mixed_path = UniversalPath::local("C:/Users\\Music/file.wav");
        assert_eq!(mixed_path.path_segments(), &["C:", "Users", "Music", "file.wav"]);
        
        // Test UNC path
        let unc_path = UniversalPath::local("\\\\server\\share\\music\\album");
        assert_eq!(unc_path.path_segments(), &["server", "share", "music", "album"]);
    }

    #[test]
    fn test_relative_to() {
        let parent = UniversalPath::local("/music/classical");
        let child = UniversalPath::local("/music/classical/beethoven/symphony_9.mp3");
        let not_child = UniversalPath::local("/videos/movies");
        
        // Test child relationship
        let relative = child.relative_to(&parent);
        assert_eq!(relative, Some(vec!["beethoven".to_string(), "symphony_9.mp3".to_string()]));
        
        // Test non-child relationship
        let not_relative = not_child.relative_to(&parent);
        assert_eq!(not_relative, None);
        
        // Test same path
        let same_relative = parent.relative_to(&parent);
        assert_eq!(same_relative, Some(vec![]));
        
        // Test parent is longer than child
        let shorter_relative = parent.relative_to(&child);
        assert_eq!(shorter_relative, None);
        
        // Test different backends
        let s3_parent = UniversalPath::from_uri_str("s3://bucket/music").unwrap();
        let local_child = UniversalPath::local("/music/classical");
        assert_eq!(local_child.relative_to(&s3_parent), None);
        
        // Test different hosts
        let host1 = UniversalPath::from_uri_str("s3://bucket1/music").unwrap();
        let host2 = UniversalPath::from_uri_str("s3://bucket2/music").unwrap();
        assert_eq!(host2.relative_to(&host1), None);
    }

    #[test]
    fn test_uri_conversion() {
        let local_path = UniversalPath::local("/music/song.mp3");
        let uri = local_path.to_uri().unwrap();
        assert_eq!(uri, "file:/music/song.mp3");
        
        let s3_path = UniversalPath::from_uri_str("s3://bucket/music/song.mp3").unwrap();
        let s3_uri = s3_path.to_uri().unwrap();
        assert_eq!(s3_uri, "s3://bucket/music/song.mp3");
    }

    #[test]
    fn test_from_str() {
        let path: UniversalPath = "s3://bucket/music/song.mp3".parse().unwrap();
        assert_eq!(path.backend(), &StorageBackend::S3);
        assert_eq!(path.host(), Some("bucket"));
        assert_eq!(path.last_segment(), Some("song.mp3"));
    }

    #[test]
    fn test_dotdot_handling() {
        let mut path = UniversalPath::local("/music/classical");
        path.append("..");
        assert_eq!(path.path(), "/music");
        
        path.append("jazz");
        assert_eq!(path.path(), "/music/jazz");
    }

    #[test]
    fn test_unicode_emoji_paths() {
        // Test multi-codepoint emojis like 👨‍👩‍👧‍👦 (family emoji)
        let family_emoji = "👨‍👩‍👧‍👦";
        let skin_tone_emoji = "👋🏽"; // Wave with medium skin tone
        let flag_emoji = "🏴󠁧󠁢󠁳󠁣󠁴󠁿"; // Scottish flag
        
        // Test local path with emojis
        let emoji_path = UniversalPath::local(&format!("/music/{}/album/{}.mp3", family_emoji, skin_tone_emoji));
        assert_eq!(emoji_path.path_segments().len(), 4);
        assert_eq!(emoji_path.path_segments()[1], family_emoji);
        let expected_filename = format!("{}.mp3", skin_tone_emoji);
        assert_eq!(emoji_path.last_segment(), Some(expected_filename.as_str()));
        println!("{:?}", emoji_path.path_segments());
        
        // Test URI roundtrip with emojis
        let uri = emoji_path.to_uri().unwrap();
        let roundtrip = UniversalPath::from_uri_str(&uri).unwrap();
        assert_eq!(emoji_path, roundtrip);
        
        // Test flag emoji in path
        let flag_path = UniversalPath::local(&format!("/countries/{}/music", flag_emoji));
        let flag_uri = flag_path.to_uri().unwrap();
        let flag_roundtrip = UniversalPath::from_uri_str(&flag_uri).unwrap();
        assert_eq!(flag_path, flag_roundtrip);
    }

    #[test]
    fn test_unicode_normalization_forms() {
        // Test different Unicode normalization forms for the same visual character
        // é can be represented as:
        // 1. NFC: single codepoint U+00E9 (é)
        // 2. NFD: decomposed as U+0065 U+0301 (e + combining acute accent)
        let nfc_path = "/café/menu.txt"; // é as single codepoint
        let nfd_path = "/cafe\u{0301}/menu.txt"; // e + combining acute accent
        
        let path_nfc = UniversalPath::local(nfc_path);
        let path_nfd = UniversalPath::local(nfd_path);
        
        // Test URI conversion
        let uri_nfc = path_nfc.to_uri().unwrap();
        let uri_nfd = path_nfd.to_uri().unwrap();
        
        // Test roundtrip conversion
        let roundtrip_nfc = UniversalPath::from_uri_str(&uri_nfc).unwrap();
        let roundtrip_nfd = UniversalPath::from_uri_str(&uri_nfd).unwrap();
        
        // Both should work, and ideally normalize to the same form
        assert!(roundtrip_nfc.path_segments()[0].contains("caf"));
        assert!(roundtrip_nfd.path_segments()[0].contains("caf"));
        
        println!("NFC URI: {}", uri_nfc);
        println!("NFD URI: {}", uri_nfd);
        println!("NFC roundtrip path: {:?}", roundtrip_nfc.path_segments());
        println!("NFD roundtrip path: {:?}", roundtrip_nfd.path_segments());
    }

    #[test]
    fn test_unicode_special_characters() {
        // Test various special Unicode characters that often cause issues
        let special_chars = vec![
            ("space_variants", "file\u{00A0}name.txt"), // Non-breaking space
            ("zero_width", "file\u{200B}name.txt"), // Zero-width space
            ("rtl_mark", "file\u{200F}name.txt"), // Right-to-left mark
            ("combining", "file\u{0300}name.txt"), // Combining grave accent
            ("surrogate", "file𝒽𝒶𝓃𝒹𝓁𝑒.txt"), // Mathematical script letters (high plane)
        ];
        
        for (test_name, filename) in special_chars {
            let path = UniversalPath::local(&format!("/test/{}", filename));
            
            // Test URI conversion
            let uri = path.to_uri().unwrap();
            
            // Test roundtrip
            let roundtrip = UniversalPath::from_uri_str(&uri).unwrap();
            
            println!("Test {}: Original: {:?}", test_name, path.path_segments());
            println!("Test {}: URI: {}", test_name, uri);
            println!("Test {}: Roundtrip: {:?}", test_name, roundtrip.path_segments());
            
            // Should at least preserve the file structure
            assert_eq!(path.path_segments().len(), roundtrip.path_segments().len());
        }
    }

    #[test]
    fn test_unicode_non_latin_scripts() {
        // Test various non-Latin scripts
        let scripts = vec![
            ("chinese", "/音乐/歌曲.mp3"),
            ("arabic", "/موسيقى/أغنية.mp3"),
            ("japanese", "/音楽/歌.mp3"),
            ("korean", "/음악/노래.mp3"),
            ("russian", "/музыка/песня.mp3"),
            ("hebrew", "/מוזיקה/שיר.mp3"),
            ("thai", "/ดนตรี/เพลง.mp3"),
            ("emoji_mix", "/🎵音楽🎶/song🎼.mp3"),
        ];
        
        for (script_name, path_str) in scripts {
            let path = UniversalPath::local(path_str);
            
            // Test URI conversion
            let uri = path.to_uri().unwrap();
            
            // Test roundtrip
            let roundtrip = UniversalPath::from_uri_str(&uri).unwrap();
            
            println!("Script {}: Original path: {}", script_name, path_str);
            println!("Script {}: URI: {}", script_name, uri);
            println!("Script {}: Roundtrip: {}", script_name, roundtrip.path());
            
            // Basic structure should be preserved
            assert_eq!(path.path_segments().len(), roundtrip.path_segments().len());
            assert!(roundtrip.last_segment().is_some());
        }
    }

    #[test]
    fn test_unicode_windows_paths() {
        // Test Unicode characters with Windows-style paths
        let windows_paths = vec![
            "C:\\用户\\音乐\\歌曲.mp3", // Chinese
            "D:\\пользователи\\музыка\\песня.mp3", // Russian
            "E:\\🎵Music🎶\\Song🎼.mp3", // Emojis
            "F:\\café\\ménü.txt", // Accented characters
        ];
        
        for path_str in windows_paths {
            let path = UniversalPath::local(path_str);
            
            // Should correctly identify drive letter
            assert!(path.path_segments().len() >= 3);
            assert!(path.path_segments()[0].ends_with(":"));
            
            // Test URI conversion
            let uri = path.to_uri().unwrap();
            
            // Test roundtrip
            let roundtrip = UniversalPath::from_uri_str(&uri).unwrap();
            
            println!("Windows path: {}", path_str);
            println!("Segments: {:?}", path.path_segments());
            println!("URI: {}", uri);
            println!("Roundtrip segments: {:?}", roundtrip.path_segments());
            
            assert_eq!(path.path_segments().len(), roundtrip.path_segments().len());
        }
    }

    #[test]
    fn test_unicode_edge_cases() {
        // Test really problematic Unicode cases
        let edge_cases = vec![
            // Null byte (should be rejected or encoded)
            ("null_byte", "/test/file\0name.txt"),
            // Control characters
            ("tab", "/test/file\tname.txt"),
            ("newline", "/test/file\nname.txt"),
            // Unicode direction overrides
            ("bidi_override", "/test/\u{202D}file\u{202C}.txt"),
            // Maximum Unicode codepoint
            ("max_unicode", "/test/file\u{10FFFF}.txt"),
            // Combining character sequences
            ("combining_seq", "/test/a\u{0300}\u{0301}\u{0302}.txt"), // a with multiple accents
        ];
        
        for (test_name, path_str) in edge_cases {
            let result = std::panic::catch_unwind(|| {
                let path = UniversalPath::local(path_str);
                let uri = path.to_uri().unwrap();
                let roundtrip = UniversalPath::from_uri_str(&uri).unwrap();
                (path, uri, roundtrip)
            });
            
            match result {
                Ok((path, uri, roundtrip)) => {
                    println!("Edge case {}: Success", test_name);
                    println!("  Original: {:?}", path.path_segments());
                    println!("  URI: {}", uri);
                    println!("  Roundtrip: {:?}", roundtrip.path_segments());
                },
                Err(_) => {
                    println!("Edge case {}: Failed (this might be expected)", test_name);
                }
            }
        }
    }

    #[test]
    fn test_unicode_s3_uri_roundtrip() {
        // Test Unicode in S3 URIs with various backends
        let unicode_s3_uris = vec![
            "s3://bucket/音乐/歌曲.mp3",
            "s3://bucket/🎵music🎶/song.mp3",
            "s3://bucket/café/ménü.txt",
            "sftp://server.com/用户/文档/файл.txt",
            "https://cdn.example.com/медиа/песня.mp3",
        ];
        
        for uri_str in unicode_s3_uris {
            let path = UniversalPath::from_uri_str(uri_str).unwrap();
            let regenerated_uri = path.to_uri().unwrap();
            let final_roundtrip = UniversalPath::from_uri_str(&regenerated_uri).unwrap();
            
            println!("Original URI: {}", uri_str);
            println!("Path segments: {:?}", path.path_segments());
            println!("Regenerated URI: {}", regenerated_uri);
            println!("Final roundtrip segments: {:?}", final_roundtrip.path_segments());
            
            // Basic structure should be preserved
            assert_eq!(path.backend(), final_roundtrip.backend());
            assert_eq!(path.host(), final_roundtrip.host());
            assert_eq!(path.path_segments().len(), final_roundtrip.path_segments().len());
        }
    }

    #[test]
    fn test_unicode_path_manipulation() {
        // Test that Unicode paths work correctly with path manipulation operations
        let base_path = UniversalPath::local("/音乐");
        
        // Test append with Unicode
        let mut path = base_path.clone();
        path.append("古典音乐");
        path.append("贝多芬");
        path.append("第九交响曲🎼.mp3");
        
        assert_eq!(path.path_segments().len(), 4);
        assert_eq!(path.last_segment(), Some("第九交响曲🎼.mp3"));
        
        // Test parent operation
        let parent = path.parent().unwrap();
        assert_eq!(parent.last_segment(), Some("贝多芬"));
        
        // Test join operation
        let new_file = parent.join("钢琴奏鸣曲🎹.flac");
        assert_eq!(new_file.last_segment(), Some("钢琴奏鸣曲🎹.flac"));
        
        // Test relative_to with Unicode
        let classical_path = UniversalPath::local("/音乐/古典音乐");
        let full_path = UniversalPath::local("/音乐/古典音乐/贝多芬/第九交响曲🎼.mp3");
        let relative = full_path.relative_to(&classical_path);
        assert_eq!(relative, Some(vec!["贝多芬".to_string(), "第九交响曲🎼.mp3".to_string()]));
    }
}
