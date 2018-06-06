# This script takes care of packaging the build artifacts that will go in the
# release zipfile

$SRC_DIR = $PWD.Path
$STAGE = [System.Guid]::NewGuid().ToString()

Set-Location $ENV:Temp
New-Item -Type Directory -Name $STAGE
New-Item -Type Directory -Name "$STAGE\Dose Response"
Set-Location $STAGE

#$ZIP = "$SRC_DIR\$($Env:CRATE_NAME)-$($Env:APPVEYOR_REPO_TAG_NAME)-$($Env:TARGET).zip"
$TARFILE = "$SRC_DIR\$($Env:CRATE_NAME)-$($Env:APPVEYOR_REPO_TAG_NAME)-$($Env:TARGET).tar"
$TARBALL = "$TARFILE.gz"

Copy-Item "$SRC_DIR\target\$($Env:TARGET)\release\dose-response.exe" '.\Dose Response\Dose Response.exe'
# NOTE(shadower): we're bundling things statically now, don't upload the full build directory anymore:
#Copy-Item "$SRC_DIR\target\$($Env:TARGET)\release\build" -Destination '.\Dose Response\' -Recurse
# Use Get-Content/Set-Content which should convert the files from LF to CR LF endings
Get-Content "$SRC_DIR\README.md" | Set-Content '.\Dose Response\README.txt'
Get-Content "$SRC_DIR\COPYING.txt" | Set-Content '.\Dose Response\LICENSE.txt'

Add-Content -Path '.\Dose Response\VERSION.txt' -Value "Version: $($Env:APPVEYOR_REPO_TAG_NAME)"
Add-Content -Path '.\Dose Response\VERSION.txt' -Value "Full Version: $($Env:CRATE_NAME)-$($Env:APPVEYOR_REPO_TAG_NAME)-$($Env:TARGET)"
Add-Content -Path '.\Dose Response\VERSION.txt' -Value "Commit: $($Env:APPVEYOR_REPO_COMMIT)"

#7z a "$ZIP" *
7z a "$TARFILE" *
7z a "$TARBALL" "$TARFILE"

# Push-AppveyorArtifact "$ZIP"
Push-AppveyorArtifact "$TARBALL"

Remove-Item *.* -Force
Set-Location ..
Remove-Item $STAGE
Set-Location $SRC_DIR
