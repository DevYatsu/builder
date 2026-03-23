# builder - Uninstall script for Windows
# https://github.com/DevYatsu/builder

$binaryName = "builder.exe"
$installDir = "$HOME\.builder"

# Colors
function Write-Info($message) { Write-Host "[INFO] $message" -ForegroundColor Cyan }
function Write-Success($message) { Write-Host "[SUCCESS] $message" -ForegroundColor Green }
function Write-Error($message) { Write-Host "[ERROR] $message" -ForegroundColor Red }

$uninstalled = $false

if (Test-Path $installDir) {
    Write-Info "Found $binaryName installation at $installDir. Removing..."
    Remove-Item -Path $installDir -Recurse -Force
    
    # Remove from User PATH
    $path = [Environment]::GetEnvironmentVariable("PATH", "User")
    $binPath = Join-Path $installDir "bin"
    if ($path -like "*$binPath*") {
        Write-Info "Removing $binPath from User PATH..."
        $newPath = $path.Replace(";$binPath", "").Replace("$binPath;", "").Replace($binPath, "")
        [Environment]::SetEnvironmentVariable("PATH", $newPath.Trim(";"), "User")
        $env:PATH = $newPath.Trim(";")
    }
    $uninstalled = $true
}

if ($uninstalled) {
    Write-Success "builder was successfully uninstalled."
} else {
    Write-Info "Could not find builder installation at $installDir."
}
