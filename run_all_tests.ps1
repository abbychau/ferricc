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
