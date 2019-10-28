If ([string]::IsNullOrEmpty($APPVEYOR_REPO_TAG)) {
    Write-Host "This is not a release tag. Running test builds."
    # Print all environment variables
    gci env:* | sort-object name

    cargo build --features "test windows-extra-features" --target $TARGET
    cargo build --features "test windows-extra-features" --target $TARGET --release
} Else {
    Write-Host "This is a release tag: $APPVEYOR_REPO_TAG. Skipping test builds."
}
