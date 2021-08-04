md "%LOCALAPPDATA%\Proxydetox"
copy /v /y "%~dp0\bin\proxydetox.exe" "%LOCALAPPDATA%\Proxydetox\proxydetox.exe"
regedit /s "%~dp0\startup.reg"
