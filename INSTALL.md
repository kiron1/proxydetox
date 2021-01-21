# Build and Install `proxydetox`

The easiest is, to use `cargo` from [rustup][rustup]. The next command will
install the `proxydetox` binary in `~/.cargo/bin`.

```sh
cargo install --git https://github.com/kiron1/proxydetox.git
```

If you have cloned this repository already, you can also do:


```sh
cargo install --path .
```

## Enable build features

To enable the Negotiate authentication method on GNU/Linux or macoOS the
[Generic Security Services Application Program Interface (GSSAPI)][gssapi] must
be enabled during compile time using the `--features gssapi` feature flag when
invoking cargo.

To enable the Negotiate authentication method on Windows the [Security Support
Provider Interface][sspi] is used and must be enable during compile time using
the `--features sspi` feature flag when invoking cargo.

[gssapi]: https://en.wikipedia.org/wiki/Generic_Security_Services_Application_Program_Interface
[sspi]: https://en.wikipedia.org/wiki/Security_Support_Provider_Interface

## Configuration

Two configuration files are needed, a) an optional `~/.netrc` file which will
store the authentication credentials for the proxy (if required by the proxy),
and b) a `proxy.pac` file in the `~/.config/proxydetox` directory.

For macOS users, the configuration is stored in the `~/Library/Application\
Support/proxydetox` directory. Please substitute accordingly.

### Negotiate authentication

To enable the Negotiate authentication the `--negotiate` flag must be added
when calling `proxydetox` or added to the configuration file
`~/.config/proxydetox/proxydetoxrc`.

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

A copy of the PAC file with the name `proxy.pac` must be places in the
`~/.config/proxydetox/` directory. The PAC file is mostly maintained by the
network administrators. The path (usually some http location in the intranet)
can be retrieved from the settings of the pre-configured internet browser.

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

[rustup]: https://rustup.rs/ "rustup.rs - The Rust toolchain installer"
