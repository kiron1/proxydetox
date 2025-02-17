function New-TemporaryDirectory {
  $parent = [System.IO.Path]::GetTempPath()
  [string] $name = [System.Guid]::NewGuid()
  $item = New-Item -ItemType Directory -Path (Join-Path $parent $name)
  return $item.FullName
}

$workdir = New-TemporaryDirectory
$root = Split-Path $PSScriptRoot
$destdir = New-Item -ItemType Directory -Path (Join-Path $workdir "proxydetox")

cargo install --path "${root}/proxydetox" --root "${destdir}" --locked --no-track --features negotiate
Copy-Item "${root}/pkg/windows/install.bat" "${destdir}/"

$pkgfile = "proxydetox-win64.zip"
Write-Output "pkgfile=${pkgfile}" | Out-File -Path "$env:GITHUB_OUTPUT"

if(Test-Path "${pkgfile}") {
  Remove-Item "${pkgfile}"
}
Get-ChildItem -Path "${workdir}" | Compress-Archive -Force -DestinationPath "${pkgfile}"

if(Test-Path "${workdir}") {
  Remove-Item -Recurse "${workdir}"
}
