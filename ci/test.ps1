If ($Env:APPVEYOR_REPO_TAG -eq "false") {
    Write-Host "This is not a release tag. Running test builds."
    # Print all environment variables
    gci env:* | sort-object name

    cargo build --target "$($Env:TARGET)" --features "test"  2>&1 | %{ "$_" }
    cargo build --target "$($Env:TARGET)" --features "test" --release  2>&1 | %{ "$_" }
} Else {
    Write-Host "This is a release tag: $Env:APPVEYOR_REPO_TAG_NAME. Skipping test builds."
}
