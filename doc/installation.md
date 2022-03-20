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

## macOS

1. Download the [`proxydetox-*-x86_64-apple-darwin.pkg`][releases] package.
2. Install using the `installer` command from macOS:
   ```sh
   sudo installer -package proxydetox-*-x86_64-apple-darwin.pkg -target /
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
