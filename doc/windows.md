# Windows

On Windows Proxydetox is _not_ a console application and therefor no console window is visible.
No console window is shown, such that Proxydetox can be launched as a background process without any disturbing windows.

The missing console windows also has the effect, that when Proxydetox is launched from the terminal no output is visible.
The help output as well as the log output is affected by this. An online version of the help output can be found in the
[command line reference](cliref.md) section.

Additionally there is the `--attach-console` options on Windows builds of Proxydetox to attach to the parrent console.
When running Proxydetox from the Windows command prompt, add the `--attach-console` optin to be able to see the output.
