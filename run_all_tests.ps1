# Function to clean up output directories
function Clear-OutputDirectories {
    param (
        [string]$TestName = $null
    )

    if ($TestName) {
        # Clean specific test files
        $ExePath = "output/bin/$TestName.exe"
        $AsmPath = "output/asm/$TestName.s"

        if (Test-Path $ExePath) {
            Write-Host "Removing existing executable: $ExePath" -ForegroundColor Yellow
            Remove-Item $ExePath -Force
        }

        if (Test-Path $AsmPath) {
            Write-Host "Removing existing assembly: $AsmPath" -ForegroundColor Yellow
            Remove-Item $AsmPath -Force
        }
    } else {
        # Clean entire directories
        Write-Host "Cleaning output directories..." -ForegroundColor Yellow

        # Check if directories exist before attempting to clean
        if (Test-Path "output/bin") {
            Get-ChildItem -Path "output/bin" -Filter "*.exe" | ForEach-Object {
                Write-Host "Removing $($_.FullName)" -ForegroundColor Yellow
                Remove-Item $_.FullName -Force
            }
        }

        if (Test-Path "output/asm") {
            Get-ChildItem -Path "output/asm" -Filter "*.s" | ForEach-Object {
                Write-Host "Removing $($_.FullName)" -ForegroundColor Yellow
                Remove-Item $_.FullName -Force
            }
        }
    }
}

# Clean output directories before running tests
Clear-OutputDirectories

# List of test files (without the .c extension)
$TestFiles = @(
    "simple",
    "factorial",
    "fact5",
    "hello_puts",
    "hello_printf",
    "simple_printf",
    "exit_code"
)

# Results tracking
$Passed = 0
$Failed = 0
$Results = @()

# Run each test
foreach ($Test in $TestFiles) {
    Write-Host "=======================================" -ForegroundColor Blue
    Write-Host "Testing: $Test" -ForegroundColor Blue
    Write-Host "=======================================" -ForegroundColor Blue

    # Clean up any existing output files for this test
    Clear-OutputDirectories -TestName $Test

    # Run the test
    $StartTime = Get-Date
    $Process = Start-Process -FilePath "powershell.exe" -ArgumentList "-File run_test.ps1 $Test" -NoNewWindow -PassThru -Wait
    $EndTime = Get-Date
    $Duration = ($EndTime - $StartTime).TotalSeconds

    # Check the result
    if ($Process.ExitCode -eq 0) {
        $Status = "PASSED"
        $Passed++
    } else {
        $Status = "FAILED (Exit code: $($Process.ExitCode))"
        $Failed++
    }

    # Store the result
    $Results += [PSCustomObject]@{
        Test = $Test
        Status = $Status
        Duration = [math]::Round($Duration, 2)
    }

    Write-Host ""
}

# Print summary
Write-Host "=======================================" -ForegroundColor Blue
Write-Host "Test Summary" -ForegroundColor Blue
Write-Host "=======================================" -ForegroundColor Blue
Write-Host "Total tests: $($TestFiles.Count)" -ForegroundColor Cyan
Write-Host "Passed: $Passed" -ForegroundColor Green
Write-Host "Failed: $Failed" -ForegroundColor Red
Write-Host ""

# Print detailed results
Write-Host "Test Results:" -ForegroundColor Yellow
$Results | Format-Table -AutoSize

# Return success if all tests passed
if ($Failed -eq 0) {
    exit 0
} else {
    exit 1
}
