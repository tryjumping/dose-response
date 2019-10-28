If ($Env:APPVEYOR_REPO_TAG -eq "false") {
    Write-Host "This is not a release tag. Running test builds."
    # Print all environment variables
    gci env:* | sort-object name

    Write-Host "Compliling target: $Env:TARGET"

    cargo build --target "$Env:TARGET" --features "test"  2>&1 | %{ "$_" }
    $native_call_success = $?
    $native_call_success
    if (-not $native_call_success) {
        throw 'error'
    }
    cargo build --target "$Env:TARGET" --features "test" --release  2>&1 | %{ "$_" }

    $native_call_success = $?
    $native_call_success
    if (-not $native_call_success) {
        throw 'error'
    }
} Else {
    Write-Host "This is a release tag: $Env:APPVEYOR_REPO_TAG_NAME. Skipping test builds."
}
