# builder - Universal install script for Windows
# https://github.com/DevYatsu/builder

$repo = "DevYatsu/builder"
$binaryName = "builder.exe"
$installDir = "$HOME\.builder\bin"

# Colors
function Write-Info($message) { Write-Host "[INFO] $message" -ForegroundColor Cyan }
function Write-Success($message) { Write-Host "[SUCCESS] $message" -ForegroundColor Green }
function Write-Error($message) { Write-Host "[ERROR] $message" -ForegroundColor Red; exit 1 }

# Detection
$arch = $env:PROCESSOR_ARCHITECTURE
if ($arch -eq "AMD64") {
    $platform = "x86_64-pc-windows-msvc"
} elseif ($arch -eq "ARM64") {
    $platform = "aarch64-pc-windows-msvc"
} else {
    Write-Error "Unsupported architecture: $arch. Please install via Cargo: cargo install builder"
}

Write-Info "Detected Windows ($arch). Fetching latest release..."

# Get latest release tag
try {
    $latestRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest"
    $latestTag = $latestRelease.tag_name
} catch {
    Write-Error "Could not fetch latest release. Please check your internet connection."
}

if (-not $latestTag) {
    Write-Error "Could not find latest release tag."
}

Write-Info "Latest version is $latestTag"

# Download URL
$filename = "builder-${platform}.zip"
$url = "https://github.com/$repo/releases/download/$latestTag/$filename"

$tmpDir = New-Item -ItemType Directory -Path "$env:TEMP\builder-install" -Force
$zipPath = Join-Path $tmpDir $filename

Write-Info "Downloading $binaryName from $url..."
try {
    Invoke-WebRequest -Uri $url -OutFile $zipPath
} catch {
    Write-Error "Download failed: $_"
}

Write-Info "Extracting..."
Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

Copy-Item -Path (Join-Path $tmpDir "builder.exe") -Destination (Join-Path $installDir "builder.exe") -Force

# Path manipulation
$path = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($path -notlike "*$installDir*") {
    Write-Info "Adding $installDir to User PATH..."
    $newPath = "$path;$installDir"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    $env:PATH = "$env:PATH;$installDir"
}

# Cleanup
Remove-Item -Path $tmpDir -Recurse -Force

Write-Success "builder has been installed successfully!"
Write-Host "You may need to restart your terminal for the changes to take effect."
Write-Host "Try it out by running: builder --help"
