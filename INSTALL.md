# Install `proxydetox`

The easiest is, to use `cargo` from [rustup][rustup]. The next command will
install the `proxydetox` binary in `~/.cargo/bin`.

```sh
cargo install --git https://github.com/kiron1/proxydetox.git
```

If you have cloned this repository already, you can also do:


```sh
cargo install --path .
```

## Configuration

Two configuration files are needed, a) an optional `~/.netrc` file which will
store the authentication credentials for the proxy (if required by the proxy),
and b) a `proxy.pac` file in the `~/.config/proxydetox` directory.

### .netrc file

The `~/.netrc` file is only needed when any of the proxies requires an
authentication header (currently only basic authentication is supported).

An example `~/.netrc` file will look as follows:

```
machine proxy.example.org
login ProxyUsername
password ProxyPassword
```

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
