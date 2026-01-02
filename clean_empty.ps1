
# 2. Cleanup Empty Folders Script
$dirs = Get-ChildItem -Path "docs" -Directory -Recurse

# Sort by length descending so we delete deep empty folders first
$dirs = $dirs | Sort-Object -Property FullName -Descending

foreach ($dir in $dirs) {
    if ((Get-ChildItem $dir.FullName | Measure-Object).Count -eq 0) {
        Write-Host "Removing empty folder: $($dir.FullName)"
        Remove-Item $dir.FullName -Force
    }
}
Write-Host "Empty folder cleanup complete."
