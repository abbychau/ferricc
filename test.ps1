param (
    [Parameter(Mandatory=$true)]
    [string]$TestName
)

# Construct the full path to the test file
$TestFile = "tests/$TestName.c"

# Check if the test file exists
if (-not (Test-Path $TestFile)) {
    Write-Error "Test file not found: $TestFile"
    exit 1
}

# Display the test file content
Write-Host "=======================================" -ForegroundColor Blue
Write-Host "Test file: $TestFile" -ForegroundColor Blue
Write-Host "=======================================" -ForegroundColor Blue
Get-Content $TestFile | ForEach-Object { Write-Host $_ }
Write-Host "=======================================" -ForegroundColor Blue
Write-Host ""

# Build the compiler if needed
Write-Host "Building the compiler..." -ForegroundColor Cyan
cargo build

# Compile the test file
Write-Host "Compiling $TestFile..." -ForegroundColor Cyan
cargo run -- $TestFile

# Check if compilation was successful
$ExePath = "output/bin/$TestName.exe"
if (-not (Test-Path $ExePath)) {
    Write-Error "Compilation failed: $ExePath not found"
    exit 1
}

# Display the generated assembly
Write-Host "=======================================" -ForegroundColor Blue
Write-Host "Generated assembly:" -ForegroundColor Blue
Write-Host "=======================================" -ForegroundColor Blue
$AsmPath = "output/asm/$TestName.s"
Get-Content $AsmPath | ForEach-Object { Write-Host $_ }
Write-Host "=======================================" -ForegroundColor Blue
Write-Host ""

# Run the compiled program
Write-Host "Running $ExePath..." -ForegroundColor Green
try {
    # Capture the output
    $Output = & $ExePath 2>&1
    $ExitCode = $LASTEXITCODE

    # Print the output
    Write-Host "=======================================" -ForegroundColor Blue
    Write-Host "Program output:" -ForegroundColor Yellow
    Write-Host "=======================================" -ForegroundColor Blue
    if ($Output) {
        Write-Host $Output
    } else {
        Write-Host "(No output)"
    }
    Write-Host "=======================================" -ForegroundColor Blue

    # Print the exit code
    Write-Host "Exit code: $ExitCode" -ForegroundColor Magenta

    # Return success if exit code is 0
    exit $ExitCode
} catch {
    Write-Error "Error running executable"
    exit 1
}
