# test.ps1
# This script runs mcp_client_caller, collects all its output, and saves it to a log file.

$LogFile = "mcp_client_caller_test.log"
$TimeoutSeconds = 60 # Set a timeout of 60 seconds

# Clear the log file if it exists
if (Test-Path $LogFile) {
    Clear-Content $LogFile
    Write-Host "Cleared previous log file: $LogFile"
}

Write-Host "Running mcp_client_caller with a timeout of $($TimeoutSeconds) seconds and logging output to $LogFile..."

# Set RUST_BACKTRACE=full for detailed panics
$env:RUST_BACKTRACE = "full"

# Start the process in the background and redirect output to separate temporary files
$TempStdoutFile = [System.IO.Path]::GetTempFileName()
$TempStderrFile = [System.IO.Path]::GetTempFileName()

$Process = Start-Process -FilePath "cargo" `
                        -ArgumentList "run --bin mcp_client_caller" `
                        -RedirectStandardOutput $TempStdoutFile `
                        -RedirectStandardError $TempStderrFile `
                        -PassThru -NoNewWindow -ErrorAction Stop # Added -ErrorAction Stop to catch Start-Process errors

# Check if process started successfully
if ($null -eq $Process) {
    Write-Host "Error: Failed to start mcp_client_caller process."
    $ExitCode = -1
} else {
    $ProcessId = $Process.Id
    Write-Host "Started mcp_client_caller with PID: $ProcessId"

    # Wait for the process to exit, with a timeout
    if ($Process.WaitForExit($TimeoutSeconds * 1000)) { # Timeout is in milliseconds
        # Process exited within the timeout
        $ExitCode = $Process.ExitCode
        Write-Host "mcp_client_caller (PID: $ProcessId) exited with code: $ExitCode."
    } else {
        # Timeout occurred
        Write-Host "Timeout: mcp_client_caller (PID: $ProcessId) did not complete within $($TimeoutSeconds) seconds. Terminating process."
        Stop-Process -Id $ProcessId -Force -ErrorAction SilentlyContinue
        $ExitCode = -1 # Indicate timeout
    }
}

# Append the collected output to the main log file
if (Test-Path $TempStdoutFile) {
    Add-Content -Path $LogFile -Value "--- STDOUT ---"
    Add-Content -Path $LogFile -Value (Get-Content $TempStdoutFile)
}
if (Test-Path $TempStderrFile) {
    Add-Content -Path $LogFile -Value "--- STDERR ---"
    Add-Content -Path $LogFile -Value (Get-Content $TempStderrFile)
}

# Clean up temporary files
Remove-Item $TempStdoutFile -ErrorAction SilentlyContinue
Remove-Item $TempStderrFile -ErrorAction SilentlyContinue

# Clear RUST_BACKTRACE environment variable after running
Remove-Item Env:RUST_BACKTRACE

if ($ExitCode -ne 0) {
    Write-Host "mcp_client_caller finished with exit code: $ExitCode. Check log for details."
} else {
    Write-Host "mcp_client_caller completed successfully. Check log for details."
}

Write-Host "Test completed. All output saved to $LogFile"