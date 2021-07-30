md "%LOCALAPPDATA%\Proxydetox"
copy /v /y proxydetox.exe "%LOCALAPPDATA%\Proxydetox\proxydetox.exe"
regedit /s "%~dp0\startup.reg"
