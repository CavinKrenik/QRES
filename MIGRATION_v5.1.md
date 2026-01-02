# QRES v5.1 Migration Guide

This guide helps you upgrade from QRES v5.0.x to v5.1.0.

---

## Table of Contents
1. [Quick Start](#quick-start)
2. [Breaking Changes](#breaking-changes)
3. [New Features](#new-features)
4. [API Changes](#api-changes)
5. [File Format Compatibility](#file-format-compatibility)
6. [Performance Considerations](#performance-considerations)

---

## Quick Start

### For End Users

**Update QRES:**
```bash
# Rust CLI
cargo install --path qres_rust --force

# Python package
pip install --upgrade qres

# GUI (QRES Studio)
cd qres-studio
npm install
npm run tauri build
```

**Test the update:**
```bash
qres-cli --version  # Should show v5.1.0
```

### For Developers

```bash
git pull origin main
cargo update
cargo test --all
```

---

## Breaking Changes

### 1. Folder Compression Behavior

**Before (v5.0.x):**
```bash
qres-cli compress -i ./my_folder -o ./output/
# Created: output/file1.txt.qres, output/file2.txt.qres, ...
```

**After (v5.1.0):**
```bash
qres-cli compress -i ./my_folder -o ./output/
# Creates: output/my_folder.qrar (single archive file)
```

**Migration:**
- Existing individual `.qres` files continue to work
- To compress files individually, compress each file separately:
  ```bash
  for file in my_folder/*; do
    qres-cli compress -i "$file" -o "output/$(basename $file).qres"
  done
  ```

### 2. GUI Folder Drop Behavior

**Before:** Dropping a folder compressed each file individually

**After:** Dropping a folder creates a `.qrar` archive

**Migration:** No code changes needed. Existing `.qres` files can still be decompressed normally.

---

## New Features

### 1. Archive Format (.qrar)

**Creating Archives:**
```rust
use qres_rust::archive::{create_archive, ArchiveOptions};

let options = ArchiveOptions {
    solid: true,                    // Enable solid compression
    level: 5,                      // Compression level
    preserve_permissions: true,     // Store file permissions
    compute_hashes: true,          // Blake3 integrity hashes
};

let manifest = create_archive("./source_dir", "./output.qrar", options)?;
println!("Compressed {} files", manifest.files.len());
```

**Extracting Archives:**
```rust
use qres_rust::archive::extract_archive;

let manifest = extract_archive("./archive.qrar", "./output_dir")?;
println!("Extracted {} files", manifest.files.len());
```

**Browsing Without Extraction:**
```rust
use qres_rust::archive::read_manifest;

let manifest = read_manifest("./archive.qrar")?;
for file in &manifest.files {
    println!("{}: {} bytes", file.path, file.original_size);
}
```

### 2. Deduplication

Deduplication happens automatically when creating archives with solid compression enabled.

**Checking Deduplication Stats:**
```rust
use qres_rust::dedup::DedupEngine;

let mut engine = DedupEngine::new(8 * 1024); // 8KB chunks
let result = engine.deduplicate(&data, 0);

println!("Original: {} bytes", result.original_size);
println!("Unique: {} bytes", result.unique_data.len());
println!("Dedup ratio: {:.2}%", result.dedup_ratio * 100.0);
```

### 3. Logistic Mixing

**Using Logistic Mix (optional):**
```rust
use qres_rust::mixer::Mixer;

let mixer = Mixer::new(None, None);
let preds = [120u8, 115, 118, 122, 119]; // Predictor outputs

// Standard linear mixing (default)
let prediction1 = mixer.mix(&preds);

// Neural logistic mixing (new in v5.1)
let prediction2 = mixer.logistic_mix(&preds);
```

> **Note**: Logistic mixing is currently experimental. The default `mix()` method is recommended for production use.

---

## API Changes

### Tauri Commands (GUI)

**New Commands:**
```javascript
// Browse archive contents
const manifest = await invoke('browse_archive', {
  archivePath: '/path/to/archive.qrar'
});

// Extract entire archive
await invoke('extract_archive', {
  archivePath: '/path/to/archive.qrar',
  outputDir: '/path/to/output'
});

// Extract single file
await invoke('extract_archive_file', {
  archivePath: '/path/to/archive.qrar',
  filePath: 'src/main.rs',
  outputPath: '/path/to/main.rs'
});
```

**Modified Commands:**
```javascript
// compress_file now creates .qrar for directories
await invoke('compress_file', {
  src: '/path/to/directory',
  dest: '/path/to/output'
}); // Creates output/directory.qrar
```

### Rust API

**No Breaking Changes** in core compression API:
```rust
// These continue to work unchanged
qres_rust::compress_chunk(&data, 0, None, None)?;
qres_rust::decompress_chunk(&compressed, 0, None)?;
qres_rust::compress_with_callback(&src, &dest, |progress, ratio, engine| {
    // Progress callback
})?;
```

---

## File Format Compatibility

### Backward Compatibility

✅ **v5.1 can decompress v5.0 files**
- All `.qres` files from v5.0.x work without changes
- Header format is unchanged

✅ **v5.0 CANNOT decompress v5.1 archives**
- `.qrar` format is new in v5.1
- Individual `.qres` files from v5.1 work in v5.0

### File Format Decision Tree

```
Do you need to share with v5.0 users?
├─ Yes → Use individual .qres files
│         (compress each file separately)
└─ No  → Use .qrar archives for better compression
```

### Format Specifications

**Individual File (.qres):**
- Magic: `QRES` (0x51 0x52 0x45 0x53)
- Version: 0x04 (unchanged from v5.0)
- Compatible with all v5.x versions

**Archive File (.qrar):**
- Magic: `QRAR` (0x51 0x52 0x41 0x52)
- Version: 0x01
- **New in v5.1** - Not compatible with v5.0

---

## Performance Considerations

### Memory Usage

**Before v5.1:**
- Decompressing required loading entire file into RAM
- Large files (>1GB) could crash

**After v5.1:**
- Streaming I/O with constant memory usage
- Can decompress files of any size

**Recommendation:** No special considerations needed. v5.1 is strictly better.

### Compression Speed

**Individual Files (.qres):**
- Same speed as v5.0
- ~50-100 MB/s on typical hardware

**Archives (.qrar):**
- Slightly slower due to solid compression
- ~30-80 MB/s (depends on file count)
- **Trade-off**: 20-50% better compression ratios

**When to use each:**
- `.qres`: Single large files, speed is critical
- `.qrar`: Multiple related files, size is critical

---

## Troubleshooting

### Issue: "Unknown codec flag" error

**Cause:** This was a bug in v5.0.x that is **fixed in v5.1**

**Solution:** Update to v5.1:
```bash
cargo install --path qres_rust --force
```

### Issue: Out of memory when decompressing

**Cause:** Another v5.0.x bug, **fixed in v5.1**

**Solution:** Update to v5.1. Memory usage is now constant regardless of file size.

### Issue: Can't open .qrar file in v5.0

**Cause:** `.qrar` format is new in v5.1

**Solutions:**
1. Update to v5.1 (recommended)
2. Or extract in v5.1 then recompress as individual `.qres` files for v5.0

### Issue: Folder compression creates individual files instead of archive

**Cause:** Using old v5.0 binary

**Check version:**
```bash
qres-cli --version
```

**Solution:** Reinstall v5.1:
```bash
cargo install --path qres_rust --force
```

---

## Best Practices

### 1. When to Use Archives (.qrar)

✅ **Good for:**
- Source code projects
- Document collections
- Backups
- Any directory with related files

❌ **Not ideal for:**
- Single large files (use `.qres`)
- Files that need frequent random access
- Sharing with v5.0 users

### 2. Archive Options

**For maximum compression:**
```rust
ArchiveOptions {
    solid: true,              // Essential for best compression
    level: 9,                 // Max compression
    preserve_permissions: true,
    compute_hashes: true,     // Adds ~2% overhead but ensures integrity
}
```

**For maximum speed:**
```rust
ArchiveOptions {
    solid: false,             // Faster but less compression
    level: 1,
    preserve_permissions: false,
    compute_hashes: false,
}
```

### 3. Integrity Verification

Always enable hash computation for important data:
```rust
options.compute_hashes = true;
```

During extraction, hashes are automatically verified. Corrupted files will fail extraction with a clear error message.

---

## Rollback Instructions

If you need to rollback to v5.0.5:

```bash
# Rust
cargo install --path qres_rust --force --vers 5.0.5

# Python
pip install qres==5.0.5

# GUI
git checkout v5.0.5
cd qres-studio
npm install
npm run tauri build
```

**Note:** After rollback, `.qrar` archives won't be accessible until you upgrade again.

---

## Getting Help

- **GitHub Issues**: [Report a bug](https://github.com/CavinKrenik/QRES/issues)
- **Discussions**: [Ask questions](https://github.com/CavinKrenik/QRES/discussions)
- **Documentation**: See `RELEASE_NOTES.md` and `CRITICAL_FIXES_V5.1.md`

---

## Summary

v5.1 is a **highly recommended** upgrade that fixes critical bugs and adds powerful archiving capabilities. The migration is straightforward with minimal breaking changes.

**Key Points:**
- ✅ Fixes critical crash and memory bugs
- ✅ Backward compatible with v5.0 `.qres` files
- ✅ New `.qrar` format provides 20-50% better compression
- ✅ Streaming I/O handles files of any size
- ⚠️ Folder compression now creates archives (intentional change)

**Recommendation:** Upgrade immediately to benefit from bug fixes.
