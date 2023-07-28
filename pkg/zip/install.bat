md "%LOCALAPPDATA%\Proxydetox\bin"
copy /v /y "%~dp0\bin\proxydetox.exe" "%LOCALAPPDATA%\Proxydetox\bin\proxydetox.exe"
reg add "HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" /v Proxydetox /t REG_SZ /d """%LOCALAPPDATA%\Proxydetox\bin\proxydetox.exe""" /f
