function New-TemporaryDirectory {
  $parent = [System.IO.Path]::GetTempPath()
  [string] $name = [System.Guid]::NewGuid()
  $item = New-Item -ItemType Directory -Path (Join-Path $parent $name)
  return $item.FullName
}

$workdir = New-TemporaryDirectory
$root = Split-Path $PSScriptRoot
$destdir = New-Item -ItemType Directory -Path (Join-Path $workdir "proxydetox")

cargo install --path "${root}/proxydetox" --root "${destdir}" --no-track
Copy-Item "${root}/pkg/windows/install.bat" "${destdir}/"
Copy-Item "${root}/pkg/windows/startup.reg" "${destdir}/"

$zipfile = "proxydetox-win64.zip"

if(Test-Path "${zipfile}") {
  Remove-Item "${zipfile}"
}
Get-ChildItem -Path "${workdir}" | Compress-Archive -Force -DestinationPath "${zipfile}"

if(Test-Path "${workdir}") {
  Remove-Item -Recurse "${workdir}"
}
