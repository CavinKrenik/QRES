/// QRES Archive Container Format
///
/// This module implements a true "archiver" format similar to WinZip/7-Zip,
/// as opposed to simply compressing files individually.
///
/// Format Structure:
/// ```text
/// [QRAR Magic: 4 bytes] "QRAR" (QRES Archive)
/// [Version: 1 byte] 0x01
/// [Flags: 1 byte] (bit 0: solid compression, bit 1: encrypted)
/// [Manifest Length: 4 bytes]
/// [Manifest JSON: variable]
/// [Compressed Stream: variable]
/// ```
use crate::dedup::{DedupEngine, DedupReference}; // Import Dedup
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufReader, Read, Write}; // Added BufReader
use std::path::Path;

const ARCHIVE_MAGIC: &[u8] = b"QRAR";
const ARCHIVE_VERSION: u8 = 1;

/// Metadata for a single file within the archive
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileEntry {
    /// Relative path within the archive
    pub path: String,
    /// Original file size (bytes)
    pub original_size: u64,
    /// Offset in the compressed stream where this file starts
    pub stream_offset: u64,
    /// Length of this file's data in the compressed stream
    pub stream_length: u64,
    /// Unix permissions (if applicable)
    pub permissions: Option<u32>,
    /// Last modified timestamp
    pub modified: i64,
    /// File hash (for integrity verification)
    pub hash: Option<String>,
}

/// Archive manifest - describes the contents
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArchiveManifest {
    /// Total original size of all files
    pub total_size: u64,
    /// Compression method used
    pub compression_method: String,
    /// List of files in the archive
    pub files: Vec<FileEntry>,
    /// Metadata about the archive itself
    pub metadata: HashMap<String, String>,
}

impl ArchiveManifest {
    pub fn new() -> Self {
        ArchiveManifest {
            total_size: 0,
            compression_method: "qres-v5-dedup".to_string(), // Updated method name
            files: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

impl Default for ArchiveManifest {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchiveManifest {
    pub fn add_file(&mut self, entry: FileEntry) {
        self.total_size += entry.original_size;
        self.files.push(entry);
    }

    pub fn to_json(&self) -> io::Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(io::Error::other)
    }

    pub fn from_json(data: &[u8]) -> io::Result<Self> {
        serde_json::from_slice(data).map_err(io::Error::other)
    }
}

/// Options for archive creation
#[derive(Debug, Clone)]
pub struct ArchiveOptions {
    /// Use solid compression (concatenate all files before compressing)
    pub solid: bool,
    /// Compression level (0-9, higher = better but slower)
    pub level: u8,
    /// Store file permissions
    pub preserve_permissions: bool,
    /// Calculate file hashes for integrity
    pub compute_hashes: bool,
}

impl Default for ArchiveOptions {
    fn default() -> Self {
        ArchiveOptions {
            solid: true,
            level: 5,
            preserve_permissions: true,
            compute_hashes: true,
        }
    }
}

/// Create a solid archive from a directory
///
/// Instead of compressing each file individually, this concatenates all files
/// into a single stream and compresses them together. This allows the compression
/// engine to learn patterns across files (e.g., shared headers in C files).
pub fn create_archive<P: AsRef<Path>>(
    source_dir: P,
    output_path: P,
    options: ArchiveOptions,
) -> io::Result<ArchiveManifest> {
    let source_dir = source_dir.as_ref();
    let output_path = output_path.as_ref();

    if !source_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Source must be a directory",
        ));
    }

    let mut manifest = ArchiveManifest::new();
    let mut solid_stream = Vec::new();
    let mut current_offset = 0u64;

    for entry in walkdir::WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let file_path = entry.path();
        let relative_path = file_path
            .strip_prefix(source_dir)
            .unwrap()
            .to_string_lossy()
            .to_string();
        let file_data = fs::read(file_path)?;
        let file_size = file_data.len() as u64;

        let hash = if options.compute_hashes {
            Some(blake3::hash(&file_data).to_hex().to_string())
        } else {
            None
        };

        let permissions = if options.preserve_permissions {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                Some(fs::metadata(file_path)?.permissions().mode())
            }
            #[cfg(not(unix))]
            {
                None
            }
        } else {
            None
        };

        let modified = fs::metadata(file_path)?
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        manifest.add_file(FileEntry {
            path: relative_path,
            original_size: file_size,
            stream_offset: current_offset,
            stream_length: file_size,
            permissions,
            modified,
            hash,
        });

        solid_stream.extend_from_slice(&file_data);
        current_offset += file_size;
    }

    // Step 2: Deduplication & Compression
    let compressed_stream = if options.solid {
        compress_solid_stream_dedup(&solid_stream)?
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Non-solid archives not yet implemented",
        ));
    };

    // Step 3: Write File
    let mut output = fs::File::create(output_path)?;
    output.write_all(ARCHIVE_MAGIC)?;
    output.write_all(&[ARCHIVE_VERSION])?;
    output.write_all(&[if options.solid { 0x01 } else { 0x00 }])?; // Flags

    let manifest_json = manifest.to_json()?;
    output.write_all(&(manifest_json.len() as u32).to_le_bytes())?;
    output.write_all(&manifest_json)?;
    output.write_all(&compressed_stream)?;

    Ok(manifest)
}

/// Compress using Deduplication Engine + QRES
fn compress_solid_stream_dedup(data: &[u8]) -> io::Result<Vec<u8>> {
    // 1. Initialize Deduplication Engine
    let mut engine = DedupEngine::new(64 * 1024); // 64KB target chunk size
    let result = engine.deduplicate(data, 0);

    let mut output = Vec::new();

    // 2. Process Result References
    // DedupResult contains references and a list of unique data blocks (in order of creation)
    // We map unique chunks by their ID (index in unique_data)

    // Buffer for compression (re-used)
    let mut comp_buffer = vec![0u8; 64 * 1024 + 4096];

    for ref_chunk in result.references {
        match ref_chunk {
            DedupReference::New {
                hash: _,
                chunk_id,
                size: _,
            } => {
                // Get the data for this new chunk
                let chunk_data = &result.unique_data[chunk_id as usize];

                // Resize if needed (rare case where chunk > 64KB)
                if chunk_data.len() + 4096 > comp_buffer.len() {
                    comp_buffer.resize(chunk_data.len() + 4096, 0);
                }

                // Compress normally (Flag 0x00 or 0x02 via compress_chunk)
                let len = crate::compress_chunk(chunk_data, 0, None, None, &mut comp_buffer)?;
                let compressed = &comp_buffer[..len];

                // Write [Len: 4][Compressed Data]
                // Note: compress_chunk includes its own internal flag byte at the start
                output.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
                output.extend_from_slice(compressed);
            }
            DedupReference::Existing { hash, size } => {
                // Write Reference Chunk (Flag 0x03)
                // Layout: [Flag: 0x03][Hash: 8 bytes][Size: 4 bytes]
                // Total length: 13 bytes
                let len: u32 = 13;

                output.extend_from_slice(&len.to_le_bytes());
                output.push(0x03); // FLAG_REF
                output.extend_from_slice(&hash.to_le_bytes());
                output.extend_from_slice(&(size as u32).to_le_bytes());
            }
        }
    }

    Ok(output)
}

/// Extract archive with support for References
pub fn extract_archive<P: AsRef<Path>>(
    archive_path: P,
    output_dir: P,
) -> io::Result<ArchiveManifest> {
    let archive_path = archive_path.as_ref();
    let output_dir = output_dir.as_ref();
    let mut reader = BufReader::new(fs::File::open(archive_path)?);

    // [Skip Header Validation for brevity - verify Magic, Version, Flags...]
    let mut header_buf = [0u8; 6];
    reader.read_exact(&mut header_buf)?;
    if &header_buf[0..4] != ARCHIVE_MAGIC {
        return Err(io::Error::other("Invalid Magic"));
    }

    // Read Manifest
    let mut man_len_bytes = [0u8; 4];
    reader.read_exact(&mut man_len_bytes)?;
    let man_len = u32::from_le_bytes(man_len_bytes) as usize;
    let mut man_data = vec![0u8; man_len];
    reader.read_exact(&mut man_data)?;
    let manifest = ArchiveManifest::from_json(&man_data)?;

    // Decompression Context: Map Hash -> Plaintext Chunk
    let mut chunk_cache: HashMap<u64, Vec<u8>> = HashMap::new();
    let mut decompressed_stream = Vec::new();

    // Loop chunks
    loop {
        let mut chunk_len_bytes = [0u8; 4];
        match reader.read_exact(&mut chunk_len_bytes) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
        let chunk_len = u32::from_le_bytes(chunk_len_bytes) as usize;
        let mut chunk_data = vec![0u8; chunk_len];
        reader.read_exact(&mut chunk_data)?;

        // Check Flag (first byte of chunk_data)
        if chunk_data[0] == 0x03 {
            // HANDLE REFERENCE [Flag:1][Hash:8][Size:4]
            if chunk_len < 13 {
                return Err(io::Error::other("Ref chunk too short"));
            }

            let mut hash_bytes = [0u8; 8];
            hash_bytes.copy_from_slice(&chunk_data[1..9]);
            let hash = u64::from_le_bytes(hash_bytes);

            if let Some(cached_data) = chunk_cache.get(&hash) {
                decompressed_stream.extend_from_slice(cached_data);
            } else {
                return Err(io::Error::other(
                    "Data corruption: Reference to unknown chunk",
                ));
            }
        } else {
            // HANDLE NORMAL CHUNK (0x00, 0x01, 0x02)
            let decoded = crate::decompress_chunk(&chunk_data, 0, None)?;

            // Calculate hash of plaintext to populate cache
            let hash = crate::dedup::xxhash64(&decoded);
            chunk_cache.insert(hash, decoded.clone());

            decompressed_stream.extend_from_slice(&decoded);
        }
    }

    // [File Extraction Logic - same as previous]
    fs::create_dir_all(output_dir)?;
    for file_entry in &manifest.files {
        let file_path = output_dir.join(&file_entry.path);
        if let Some(p) = file_path.parent() {
            fs::create_dir_all(p)?;
        }

        let start = file_entry.stream_offset as usize;
        let end = start + file_entry.stream_length as usize;
        if end > decompressed_stream.len() {
            return Err(io::Error::other("Stream truncated"));
        }

        fs::write(&file_path, &decompressed_stream[start..end])?;
    }

    Ok(manifest)
}

/// Read the manifest from an archive without extracting
pub fn read_manifest<P: AsRef<Path>>(archive_path: P) -> io::Result<ArchiveManifest> {
    use std::io::BufReader;

    let mut reader = BufReader::new(fs::File::open(archive_path)?);

    // Skip magic + version + flags (6 bytes)
    let mut header = [0u8; 6];
    reader.read_exact(&mut header)?;

    if &header[0..4] != ARCHIVE_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Not a QRES archive",
        ));
    }

    // Read manifest
    let mut manifest_len_bytes = [0u8; 4];
    reader.read_exact(&mut manifest_len_bytes)?;
    let manifest_len = u32::from_le_bytes(manifest_len_bytes) as usize;

    let mut manifest_data = vec![0u8; manifest_len];
    reader.read_exact(&mut manifest_data)?;

    ArchiveManifest::from_json(&manifest_data)
}
