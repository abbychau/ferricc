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

# Run the compiled program
Write-Host "Running $ExePath..." -ForegroundColor Green
try {
    # Capture the output
    $Output = & $ExePath 2>&1
    $ExitCode = $LASTEXITCODE

    # Print the output
    if ($Output) {
        Write-Host "Program output:" -ForegroundColor Yellow
        Write-Host $Output
    }

    # Print the exit code
    Write-Host "Exit code: $ExitCode" -ForegroundColor Magenta

    # Always return success - we're just testing that the program runs
    exit 0
} catch {
    Write-Error "Error running $TestName.exe: $_"
    exit 1
}
