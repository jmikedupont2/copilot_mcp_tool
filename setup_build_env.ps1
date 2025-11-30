# setup_build_env.ps1

# --- 1. Define VCPKG_ROOT path ---
$vcpkgRoot = Join-Path $PSScriptRoot "vcpkg"
if (-not (Test-Path $vcpkgRoot -PathType Container)) {
    Write-Error "vcpkg submodule not found at $vcpkgRoot. Please ensure it is correctly checked out."
    exit 1
}

# --- 2. Define sccache.exe path ---
$sccacheExePath = Join-Path $PSScriptRoot "sccache_meta_introspector\target\release\sccache.exe"
$sccacheDir = Split-Path -Parent $sccacheExePath
if (-not (Test-Path $sccacheExePath -PathType Leaf)) {
    Write-Error "sccache.exe not found at $sccacheExePath. Please ensure it was built successfully."
    exit 1
}

# --- 3. Set VCPKG_ROOT permanently for the current user ---
Write-Host "Setting VCPKG_ROOT environment variable to $vcpkgRoot for current user..."
[System.Environment]::SetEnvironmentVariable('VCPKG_ROOT', $vcpkgRoot, 'User')
Write-Host "VCPKG_ROOT set."

# --- 4. Add sccache directory to PATH permanently for the current user ---
Write-Host "Adding $sccacheDir to user's PATH environment variable..."
$currentPath = [System.Environment]::GetEnvironmentVariable('Path', 'User')
if ($currentPath -notlike "*$sccacheDir*") {
    [System.Environment]::SetEnvironmentVariable('Path', "$currentPath;$sccacheDir", 'User')
    Write-Host "$sccacheDir added to PATH."
} else {
    Write-Host "$sccacheDir is already in PATH."
}

# --- 5. Set RUSTC_WRAPPER permanently for the current user ---
Write-Host "Setting RUSTC_WRAPPER environment variable to sccache for current user..."
[System.Environment]::SetEnvironmentVariable('RUSTC_WRAPPER', 'sccache', 'User')
Write-Host "RUSTC_WRAPPER set."

# --- 6. Start the sccache server ---
Write-Host "Starting sccache server..."
Start-Process -FilePath $sccacheExePath -ArgumentList "--start-server" -NoNewWindow
Start-Sleep -Seconds 2 # Give sccache a moment to start
& $sccacheExePath --show-stats # Show stats to confirm it's running
Write-Host "sccache server started."

# --- 7. Install vcpkg packages for RustDesk ---
Write-Host ""
Write-Host "Installing vcpkg packages for RustDesk (this may take a long time)..."
Push-Location $vcpkgRoot

# List of core packages to install
$vcpkgPackages = @(
    "aom",
    "libjpeg-turbo",
    "opus",
    "libvpx",
    "libyuv",
    "mfx-dispatch",
    "ffmpeg"
)

foreach ($package in $vcpkgPackages) {
    Write-Host "Installing $package:x64-windows-static..."
    & "$vcpkgRoot\vcpkg.exe" install "$package`:x64-windows-static"
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to install vcpkg package: $package:x64-windows-static. Please check the vcpkg output above."
        Pop-Location
        exit 1
    }
}

Pop-Location
Write-Host "vcpkg package installation complete."

Write-Host ""
Write-Host "--------------------------------------------------------"
Write-Host "SETUP COMPLETE!"
Write-Host "For changes to environment variables (VCPKG_ROOT, PATH, RUSTC_WRAPPER) to take full effect,"
Write-Host "PLEASE RESTART YOUR TERMINAL (PowerShell, VS Code, etc.)."
Write-Host "After restarting, you can run 'cargo build' to utilize sccache and vcpkg."
Write-Host "--------------------------------------------------------"