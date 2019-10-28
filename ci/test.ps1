If ($Env:APPVEYOR_REPO_TAG -eq "false") {
    Write-Host "This is not a release tag. Running test builds."
    # Print all environment variables
    gci env:* | sort-object name

    Write-Host "$($Env:TARGET)"
    Write-Host "$Env:TARGET"
    Write-Host "$TARGET"
    Write-Host "%TARGET%"

    cargo build --target "$($Env:TARGET)" --features "test windows-extra-features"
    cargo build --target "$($Env:TARGET)" --features "test windows-extra-features" --release
} Else {
    Write-Host "This is a release tag: $Env:APPVEYOR_REPO_TAG_NAME. Skipping test builds."
}
