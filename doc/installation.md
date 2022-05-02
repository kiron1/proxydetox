# Install Proxydetox

The easiest is, to use the download and install a pre-build package from the
[releases page][releases].

Pre build binaries are available for the following platforms:

- Linux
- macOS
- Windows

## Linux

1. Download the [`proxydetox-*-x86_64-linux.deb`][releases] Debian package.
2. Install using the `dpkg` command from on Debian based platforms:
   ```sh
   sudo dpkg --install proxydetox-*-x86_64-linux.deb
   ```

### Register Proxydetox as systemd user daemon

The Debian package comes with systemd service file.

Enabling Proxydetox and starting it, can be done with the following commands:

```sh
systemctl --user daemon-reload
systemctl --user enable proxydetox
systemctl --user start proxydetox
```

To disable and stop it please use:

```sh
systemctl --user stop proxydetox
systemctl --user disable proxydetox
```

## macOS

1. Download the [`proxydetox-*-x86_64-apple-darwin.pkg`][releases] package.
2. Install using the `installer` command from macOS:
   ```sh
   sudo installer -package proxydetox-*-x86_64-apple-darwin.pkg -target /
   ```

### Register proxydetox as LaunchAgent

The above steps installed a LaunchAgent for proxydetox named
`/Library/LaunchAgents/cc.colorto.proxydetox.plist`.

**Info**: Since the `installer` command is run as the `root` user (to be able to write the files to
the system directories) it cannot know for which user the launch agent needs to be enabled.

The following steps need to be executed as the user who wants to use proxydetox (i.e. _not_ as
_root_).

```sh
{{#include launchctl.sh:install}}
```

To revert the above changes (i.e. you want to uninstall proxydetox), run the following commands:

```sh
{{#include launchctl.sh:uninstall}}
```

## Windows

1. Download the [`proxydetox-win64.zip`][releases] zip file.
2. Unzip the archive.
3. Run the included [`install.bat`][installbat] file.

## From sources

To build Proxydetox from sources, please refer the
[Build Proxydetox](./build.md) section.

[releases]: https://github.com/kiron1/proxydetox/releases/latest "Proxydetox releases"
[installbat]: https://github.com/kiron1/proxydetox/blob/main/pkg/windows/install.bat "install.bat"
