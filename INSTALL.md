# Build and Install `proxydetox`

The easiest is, to use the download and install a pre-build package from the
[realeases page][releases]. To build Proxydetox fron sources, please refer
the [BUILD.md](./BUILD.md) file.

[releases]: https://github.com/kiron1/proxydetox/releases "Proxydetox releases"

## Configuration

### File locations

The following files are read during startup by Proxydetox. These files can be
used to tweak the behaviour of Proxydetox.

- `proxydetoxrc`: Main configuration file.
- `proxy.pac`: The PAC file, which defines the rules to select the correct
  upstream proxy.
- `.netrc`: Contains the authentication information when `--negotiate` is
   _not_ used.

The configuration files are searched at the following locations:

The `.netrc` file is expected to be located in the `HOME` directory on all platforms.
If needed, the location can be specified via the `--netrc-file` flag when
invoking Proxydetox.

For `proxydetoxrc` and `proxy.pac` at a user configurable location and system
wide location is searched.

For the different platforms, the user configurable location is as follows:

- `~/.config/proxydetox/` (POSIX-compliant systems like **BSD** and **Linux**)
- `~/Library/Application\ Support/proxydetox` (**macOS**)
- `%APPDATA%` (**Windows**)

For the different platforms, the user system wide location is as follows:

POSIX-compliant systems like **BSD**, **macOS**, and **Linux**:

- `/usr/etc/proxydetox`
- `/usr/local/etc/proxydetox/`

Windows:

- `.` (the directory of the the executable)

### Negotiate authentication

To enable the Negotiate authentication the `--negotiate` flag must be added
when calling `proxydetox` or added to the `proxydetoxrc` configuration file.

### Basic authentication

When the basic authentication shall be used (default), the credentials are read
from the `~/.netrc` file.  An example `~/.netrc` file will look as follows:

```
machine proxy.example.org
login ProxyUsername
password ProxyPassword
```

The basic authentication is insecure, since it required to store the
password in clear text on disk and the password is transferred unencrypted.

### Proxy Auto-Configuration (PAC) file

A copy of the PAC file `proxy.pac` must be places in on of directories searched
by Proxydetox or specified via the `--pac-file` option. The PAC file is usually
maintained by the network administrators. The path (usually some http location
in the intranet) can be retrieved from the settings of the pre-configured
internet browser.

## Automatically start proxydetox with a user session

To automatically start `proxydetox` when an user session is active, we can
register it with `systemd(8)` on Linux or `launchd(8)` on macOS.

### Linux

```sh
cp pkg/proxydetox.service ~/.config/systemd/user/proxydetox.service
systemctl --user daemon-reload
systemctl --user enable proxydetox.service
systemctl --user start proxydetox.service
```

### macOS

```sh
cp pkg/com.github.kiron1.proxydetox.plist ~/Library/LaunchAgents/
launchctl load -w -F ~/Library/LaunchAgents/com.github.kiron1.proxydetox.plist
launchctl start com.github.kiron1.proxydetox
```
