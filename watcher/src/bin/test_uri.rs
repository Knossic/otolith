use std::env;

use watcher::{
    open_storage_for, EntryKind, EntryMetadata, Storage, StorageCapabilities, StorageError,
    UniversalPath, UniversalPathError,
};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: test_uri <URI-or-local-path>\n\nExamples:\n  test_uri file:/etc/hosts\n  test_uri /etc/hosts\n  test_uri s3://bucket/path/to/file\n  test_uri sftp://host:22/path\n"
        );
        std::process::exit(1);
    }

    let input = &args[1];

    let upath = match parse_input_to_universal_path(input) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Failed to parse input into UniversalPath: {}", format_universal_path_error(&e));
            std::process::exit(2);
        }
    };

    println!("Input: {}", input);
    println!("Parsed as UniversalPath: {}", upath);
    print_universal_path_details(&upath);

    match open_storage_for(&upath) {
        Ok(storage) => {
            println!("\nStorage backend: {:?}", storage.backend());
            let caps = storage.capabilities();
            print_capabilities(&caps);

            // Try stat
            if caps.can_stat {
                match storage.stat(&upath).await {
                    Ok(meta) => {
                        println!("\nstat():");
                        print_entry_metadata(&meta);

                        // If it's a directory and we can list, try listing a few entries
                        if meta.kind == EntryKind::Directory && caps.can_list {
                            println!("\nlist():");
                            match storage.list(&upath).await {
                                Ok(children) => {
                                    if children.is_empty() {
                                        println!("  (empty)");
                                    } else {
                                        let max_show = 50usize;
                                        for (idx, child) in children.iter().take(max_show).enumerate() {
                                            println!("  [{}] {}", idx, child);
                                        }
                                        if children.len() > max_show {
                                            println!("  ... and {} more", children.len() - max_show);
                                        }
                                    }
                                }
                                Err(err) => {
                                    println!("  list() error: {}", format_storage_error(&err));
                                }
                            }
                        }

                        // If it's a file and we can read, try reading a small portion
                        if meta.kind == EntryKind::File && (caps.can_read_range || caps.can_read) {
                            println!("\nread():");
                            let preview_len: u64 = 64 * 1024; // 64 KiB preview
                            if caps.can_read_range {
                                match storage.read_range(&upath, 0..preview_len).await {
                                    Ok(buf) => print_read_preview(&buf),
                                    Err(err) => println!("  read_range() error: {}", format_storage_error(&err)),
                                }
                            } else {
                                match storage.read(&upath).await {
                                    Ok(mut buf) => {
                                        if buf.len() as u64 > preview_len {
                                            buf.truncate(preview_len as usize);
                                        }
                                        print_read_preview(&buf);
                                    }
                                    Err(err) => println!("  read() error: {}", format_storage_error(&err)),
                                }
                            }
                        }
                    }
                    Err(err) => {
                        println!("\nstat() error: {}", format_storage_error(&err));
                        // If stat fails but list or read capabilities claim true, try them anyway
                        if caps.can_list {
                            println!("\nlist() attempt after stat failure:");
                            match storage.list(&upath).await {
                                Ok(children) => {
                                    if children.is_empty() {
                                        println!("  (empty)");
                                    } else {
                                        for (idx, child) in children.iter().enumerate().take(20) {
                                            println!("  [{}] {}", idx, child);
                                        }
                                    }
                                }
                                Err(e2) => println!("  list() error: {}", format_storage_error(&e2)),
                            }
                        }
                        if caps.can_read || caps.can_read_range {
                            println!("\nread() attempt after stat failure:");
                            if caps.can_read_range {
                                match storage.read_range(&upath, 0..4096).await {
                                    Ok(buf) => print_read_preview(&buf),
                                    Err(e2) => println!("  read_range() error: {}", format_storage_error(&e2)),
                                }
                            } else {
                                match storage.read(&upath).await {
                                    Ok(buf) => print_read_preview(&buf),
                                    Err(e2) => println!("  read() error: {}", format_storage_error(&e2)),
                                }
                            }
                        }
                    }
                }
            } else {
                println!("\nstat(): capability not supported");
            }

            // Glob is optional; attempt only if claimed
            if caps.can_glob {
                println!("\nglob(): capability claimed but no pattern provided; skipping");
            } else {
                println!("\nglob(): capability not supported");
            }
        }
        Err(err) => {
            println!("\nFailed to open storage for path: {}", format_storage_error(&err));
        }
    }
}

fn parse_input_to_universal_path(input: &str) -> Result<UniversalPath, UniversalPathError> {
    // Heuristic: if input looks like a URI, parse as URI; otherwise treat as local path
    if input.contains("://") || input.starts_with("file:") {
        UniversalPath::from_uri_str(input)
    } else {
        Ok(UniversalPath::local(input))
    }
}

fn print_universal_path_details(upath: &UniversalPath) {
    println!("\nUniversalPath details:");
    println!("  backend: {:?}", upath.backend());
    println!("  host: {:?}", upath.host());
    println!("  port: {:?}", upath.port());
    println!("  path: {}", upath.path());
    println!("  is_root: {}", upath.is_root());
    println!("  last_segment: {:?}", upath.last_segment());
    println!("  extension: {:?}", upath.extension());
    match upath.to_uri() {
        Ok(uri) => println!("  to_uri(): {}", uri),
        Err(e) => println!("  to_uri() error: {}", format_universal_path_error(&e)),
    }
    println!("  path_segments ({}): {:?}", upath.path_segments().len(), upath.path_segments());
}

fn print_capabilities(caps: &StorageCapabilities) {
    println!("\nStorage capabilities:");
    println!("  can_stat: {}", caps.can_stat);
    println!("  can_read: {}", caps.can_read);
    println!("  can_read_range: {}", caps.can_read_range);
    println!("  can_list: {}", caps.can_list);
    println!("  can_glob: {}", caps.can_glob);
}

fn print_entry_metadata(meta: &EntryMetadata) {
    println!("  kind: {:?}", meta.kind);
    println!("  size_bytes: {:?}", meta.size_bytes);
    println!("  modified_at: {:?}", meta.modified_at);
    println!("  created_at: {:?}", meta.created_at);
}

fn print_read_preview(buf: &[u8]) {
    println!("  preview bytes: {} (showing up to 512 bytes)", buf.len());
    let show = buf.len().min(512);
    let mut hex = String::new();
    for (i, b) in buf[..show].iter().enumerate() {
        if i > 0 {
            if i % 16 == 0 {
                hex.push('\n');
            } else {
                hex.push(' ');
            }
        }
        hex.push_str(&format!("{:02X}", b));
    }
    if !hex.is_empty() {
        println!("  hex:\n{}", indent_block(&hex, 4));
    }
    // Also try to show safe UTF-8 preview
    match std::str::from_utf8(&buf[..show]) {
        Ok(s) => println!("  utf8: \n{}", indent_block(&truncate_str(s, 512), 4)),
        Err(_) => println!("  utf8: (invalid UTF-8)"),
    }
}

fn indent_block(s: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    s.lines()
        .map(|line| format!("{}{}", pad, line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut out = s[..max_len].to_string();
        out.push_str("…");
        out
    }
}

fn format_storage_error(err: &StorageError) -> String {
    format!("{}", err)
}

fn format_universal_path_error(err: &UniversalPathError) -> String {
    format!("{}", err)
}


