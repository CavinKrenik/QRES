/// QRES Archive Container Format
///
/// This module implements a true "archiver" format similar to WinZip/7-Zip,
/// as opposed to simply compressing files individually.
///
/// Format Structure:
/// ```
/// [QRAR Magic: 4 bytes] "QRAR" (QRES Archive)
/// [Version: 1 byte] 0x01
/// [Flags: 1 byte] (bit 0: solid compression, bit 1: encrypted)
/// [Manifest Length: 4 bytes]
/// [Manifest JSON: variable]
/// [Compressed Stream: variable]
/// ```
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
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
            compression_method: "qres-v5-solid".to_string(),
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

    // Step 1: Walk directory and build manifest
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

        // Read file
        let file_data = fs::read(file_path)?;
        let file_size = file_data.len() as u64;

        // Compute hash if requested
        let hash = if options.compute_hashes {
            Some(blake3::hash(&file_data).to_hex().to_string())
        } else {
            None
        };

        // Get permissions
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

        // Get modification time
        let modified = fs::metadata(file_path)?
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Add to manifest
        let file_entry = FileEntry {
            path: relative_path,
            original_size: file_size,
            stream_offset: current_offset,
            stream_length: file_size, // Will be updated after compression
            permissions,
            modified,
            hash,
        };

        manifest.add_file(file_entry);

        // Append to solid stream
        solid_stream.extend_from_slice(&file_data);
        current_offset += file_size;
    }

    // Step 2: Compress the solid stream using QRES
    let compressed_stream = if options.solid {
        // Use QRES chunk compression for the entire concatenated stream
        compress_solid_stream(&solid_stream)?
    } else {
        // Fallback: individual file compression (not implemented yet)
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Non-solid archives not yet implemented",
        ));
    };

    // Step 3: Write archive file
    let mut output = fs::File::create(output_path)?;

    // Write header
    output.write_all(ARCHIVE_MAGIC)?;
    output.write_all(&[ARCHIVE_VERSION])?;

    let flags = if options.solid { 0x01 } else { 0x00 };
    output.write_all(&[flags])?;

    // Write manifest
    let manifest_json = manifest.to_json()?;
    output.write_all(&(manifest_json.len() as u32).to_le_bytes())?;
    output.write_all(&manifest_json)?;

    // Write compressed stream
    output.write_all(&compressed_stream)?;

    Ok(manifest)
}

/// Compress a solid stream using QRES chunked compression
fn compress_solid_stream(data: &[u8]) -> io::Result<Vec<u8>> {
    const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks

    let mut output = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        let chunk_end = (offset + CHUNK_SIZE).min(data.len());
        let chunk = &data[offset..chunk_end];

        // Compress chunk using QRES
        let compressed = crate::compress_chunk(chunk, 0, None, None)?;

        // Write chunk length + data
        output.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        output.extend_from_slice(&compressed);

        offset = chunk_end;
    }

    Ok(output)
}

/// Extract an archive
pub fn extract_archive<P: AsRef<Path>>(
    archive_path: P,
    output_dir: P,
) -> io::Result<ArchiveManifest> {
    use std::io::BufReader;

    let archive_path = archive_path.as_ref();
    let output_dir = output_dir.as_ref();

    let mut reader = BufReader::new(fs::File::open(archive_path)?);

    // Read and validate magic
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if magic != ARCHIVE_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Not a QRES archive",
        ));
    }

    // Read version
    let mut version = [0u8; 1];
    reader.read_exact(&mut version)?;
    if version[0] != ARCHIVE_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsupported archive version: {}", version[0]),
        ));
    }

    // Read flags
    let mut flags = [0u8; 1];
    reader.read_exact(&mut flags)?;
    let is_solid = (flags[0] & 0x01) != 0;

    // Read manifest
    let mut manifest_len_bytes = [0u8; 4];
    reader.read_exact(&mut manifest_len_bytes)?;
    let manifest_len = u32::from_le_bytes(manifest_len_bytes) as usize;

    let mut manifest_data = vec![0u8; manifest_len];
    reader.read_exact(&mut manifest_data)?;

    let manifest = ArchiveManifest::from_json(&manifest_data)?;

    // Decompress solid stream
    let mut decompressed_stream = Vec::new();

    if is_solid {
        // Read and decompress all chunks
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

            let decoded = crate::decompress_chunk(&chunk_data, 0, None)?;
            decompressed_stream.extend_from_slice(&decoded);
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Non-solid archives not yet supported",
        ));
    }

    // Extract individual files from the stream
    fs::create_dir_all(output_dir)?;

    for file_entry in &manifest.files {
        let file_path = output_dir.join(&file_entry.path);

        // Create parent directories
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Extract file data from stream
        let start = file_entry.stream_offset as usize;
        let end = start + file_entry.stream_length as usize;

        if end > decompressed_stream.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("File '{}' extends beyond stream", file_entry.path),
            ));
        }

        let file_data = &decompressed_stream[start..end];

        // Verify hash if present
        if let Some(ref expected_hash) = file_entry.hash {
            let actual_hash = blake3::hash(file_data).to_hex().to_string();
            if &actual_hash != expected_hash {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Hash mismatch for '{}'", file_entry.path),
                ));
            }
        }

        // Write file
        fs::write(&file_path, file_data)?;

        // Restore permissions
        #[cfg(unix)]
        if let Some(mode) = file_entry.permissions {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(mode);
            fs::set_permissions(&file_path, perms)?;
        }
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
