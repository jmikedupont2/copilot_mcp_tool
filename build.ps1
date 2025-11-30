# build.ps1

# --- 1. Define VCPKG_ROOT path for current session ---
$vcpkgRoot = Join-Path $PSScriptRoot "vcpkg"
if (-not (Test-Path $vcpkgRoot -PathType Container)) {
    Write-Error "vcpkg submodule not found at $vcpkgRoot. Please ensure it is correctly checked out."
    exit 1
}
$env:VCPKG_ROOT = $vcpkgRoot
Write-Host "VCPKG_ROOT set to: $env:VCPKG_ROOT (for this session)"

# --- 2. Define sccache.exe path ---
$sccacheExePath = Join-Path $PSScriptRoot "sccache_meta_introspector\target\release\sccache.exe"
$sccacheDir = Split-Path -Parent $sccacheExePath
if (-not (Test-Path $sccacheExePath -PathType Leaf)) {
    Write-Error "sccache.exe not found at $sccacheExePath. Please ensure it was built successfully using 'cargo build --release' in sccache_meta_introspector directory."
    exit 1
}

# --- 3. Add sccache directory to PATH for current session ---
# This ensures sccache.exe can be found by cargo
$env:Path = "$sccacheDir;$env:Path"
Write-Host "sccache directory added to PATH for this session."

# --- 4. Set RUSTC_WRAPPER for current session ---
$env:RUSTC_WRAPPER = "sccache"
Write-Host "RUSTC_WRAPPER set to: $env:RUSTC_WRAPPER (for this session)"

Write-Host ""
Write-Host "Ensuring sccache server is running..."
# Stop any existing sccache server
& $sccacheExePath --stop-server | Out-Null
Start-Sleep -Seconds 1 # Give it a moment to stop
# Start sccache server
& $sccacheExePath --start-server | Out-Null
Start-Sleep -Seconds 2 # Give it a moment to start

Write-Host "sccache status:"
& $sccacheExePath --show-stats

Write-Host ""
Write-Host "Starting full project build with cargo build --release..."
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "--------------------------------------------------------"
    Write-Host "BUILD COMPLETED SUCCESSFULLY!"
    Write-Host "--------------------------------------------------------"
} else {
    Write-Error "BUILD FAILED. Check the output above for errors."
}

# Optional: Stop sccache server after build if desired, or let it run
# & $sccacheExePath --stop-server