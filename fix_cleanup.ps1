
# 1. Recovery: If 'docs' is a file (the misplaced whitepaper), rename it back
if ((Test-Path "docs" -PathType Leaf)) {
    Write-Host "Recovering 'docs' file back to 'WHITEPAPER.md'..."
    Move-Item -Path "docs" -Destination "WHITEPAPER.md" -Force
}

# 2. Creation: Create the actual directory structure
New-Item -ItemType Directory -Force -Path "docs"
New-Item -ItemType Directory -Force -Path "docs/benchmarks"
New-Item -ItemType Directory -Force -Path "docs/guides"
New-Item -ItemType Directory -Force -Path "docs/archive"
New-Item -ItemType Directory -Force -Path "docs/releases"

# 3. Migration: Move Root Markdown Files explicitly
Write-Host "Moving root markdown files..."

$moves = @{
    "BENCHMARK_v5.md"         = "docs/benchmarks/"
    "WHITEPAPER.md"           = "docs/"
    "ROADMAP.md"              = "docs/"
    "MIGRATION_v5.1.md"       = "docs/guides/"
    "RELEASE_NOTES.md"        = "docs/releases/"
    "RELEASE_PACKAGE_v5.1.md" = "docs/releases/"
    "CONTRIBUTING.md"         = "docs/"
    "CHANGELOG.md"            = "docs/"
}

foreach ($file in $moves.Keys) {
    if (Test-Path $file) {
        Move-Item -Path $file -Destination $moves[$file] -Force
        Write-Host "Moved $file"
    }
}

# 4. Migration: Move qres-studio Docs
Write-Host "Moving qres-studio docs..."
if (Test-Path "qres-studio/P2P_IMPLEMENTATION.md") {
    Move-Item -Path "qres-studio/P2P_IMPLEMENTATION.md" -Destination "docs/guides/" -Force
}
if (Test-Path "qres-studio/STREAMLINED_RELEASE.md") {
    Move-Item -Path "qres-studio/STREAMLINED_RELEASE.md" -Destination "docs/guides/" -Force
}

# 5. Cleanup: Remove specific loose/legacy folders if explicitly requested (e.g. DOCS folder if it still persists as a folder)
if (Test-Path "DOCS" -PathType Container) {
    # Move anything left inside?
    Get-ChildItem "DOCS" | Move-Item -Destination "docs/archive/" -Force -ErrorAction SilentlyContinue
    Remove-Item "DOCS" -Recurse -Force
}

Write-Host "Cleanup complete."
