# cleanup_repo.ps1
# QRES Clean Slate Restructuring Script

Write-Host "Starting QRES Repository Cleanup..." -ForegroundColor Cyan

# 1. Create Target Directories
$dirs = @(
    "legacy/v1_python",
    "python",
    "benchmarks",
    "tests/data",
    "docs" # Will handle DOCS rename safely
)

foreach ($d in $dirs) {
    if (-not (Test-Path $d)) {
        New-Item -ItemType Directory -Path $d -Force | Out-Null
        Write-Host "Created: $d" -ForegroundColor Green
    }
}

# 2. Move Legacy Python Scripts
$legacyFiles = @(
    "QRS.py", "analyzer.py", "encoder.py", "main.py", 
    "qres_chunked_compressor.py", "qres_compressor.py", 
    "qres_core.py", "qres_file.py", "qres_image.py", "qres_text.py"
)

foreach ($file in $legacyFiles) {
    if (Test-Path $file) {
        Move-Item -Path $file -Destination "legacy/v1_python/" -Force
        Write-Host "Moved Legacy: $file" -ForegroundColor Yellow
    }
}

# 3. Move Test Data
$dataFiles = @("*.csv", "*.qres", "*.txt")
foreach ($pattern in $dataFiles) {
    Get-ChildItem -Path . -Filter $pattern | ForEach-Object {
        Move-Item -Path $_.FullName -Destination "tests/data/" -Force
        Write-Host "Moved Data: $($_.Name)" -ForegroundColor Yellow
    }
}

# 4. Handle Docs Rename (DOCS -> docs)
if (Test-Path "DOCS") {
    # Windows is case-insensitive, so simple rename might fail if target exists.
    # We move contents if 'docs' created above, or rename if not.
    Get-ChildItem "DOCS" | Move-Item -Destination "docs" -Force
    Remove-Item "DOCS" -Force
    Write-Host "Renamed DOCS to docs" -ForegroundColor Yellow
}

# 5. Elevate Python Package & Benchmarks from qres_rust/
# I previously created them in qres_rust/python and qres_rust/benchmarks.
# We want them at root now.

if (Test-Path "qres_rust/python/qres") {
    if (Test-Path "python/qres") { Remove-Item "python/qres" -Recurse -Force }
    Move-Item -Path "qres_rust/python/qres" -Destination "python/" -Force
    Write-Host "Elevated python/qres from subdirectory" -ForegroundColor Green
}

if (Test-Path "qres_rust/benchmarks") {
    Copy-Item -Path "qres_rust/benchmarks/*" -Destination "benchmarks/" -Force -Recurse
    Remove-Item "qres_rust/benchmarks" -Recurse -Force
    Write-Host "Elevated benchmarks from subdirectory" -ForegroundColor Green
}

# Cleanup Empty Dirs (Optional)
if (Test-Path "qres_rust/python") { Remove-Item "qres_rust/python" -Force -Recurse -ErrorAction SilentlyContinue }

Write-Host "Cleanup Complete! Repository Structure Updated." -ForegroundColor Cyan
