# Proxydetox as user service

Register Proxydetox as a user service such that it is automatically started by
the system when the user is logged in.

**Note:** This steps are only required when using `cargo install`. When installing the provided
packages this steps are not necessary.

## Automatically start Proxydetox with a user session

To automatically start `proxydetox` when an user session is active, we can
register it with `systemd(8)` on Linux or `launchd(8)` on macOS.

### Linux

Create a file `~/.config/systemd/user/proxydetox.service`, you can use
[`debian/proxydetox.service`][service] as template, but make sure to update the
[`ExecStart`][execstart] part with an _absolute_ path.

To finally enable the service, us the following commands:

```sh
systemctl --user daemon-reload
systemctl --user enable proxydetox.service
systemctl --user start proxydetox.service
```

### macOS

Create a file `~/Library/LaunchAgents/cc.colorto.proxydetox.plist`, you can use
[`cc.colorto.proxydetox.plist`][plist] as template, but make sure to update the
`ProgramArguments` value with an _absolute_ path.

The following steps need to be executed as the user who wants to use proxydetox (i.e. _not_ as
_root_).

```sh
{{#include launchctl.sh:install}}
```

To revert the above changes (i.e. you want to uninstall proxydetox), run the following commands:

```sh
{{#include launchctl.sh:uninstall}}
```

[service]: https://github.com/kiron1/proxydetox/blob/main/debian/proxydetox.service "proxydetox.service file"
[execstart]: https://man7.org/linux/man-pages/man5/systemd.service.5.html "man 5 systemd.service"
[plist]: https://github.com/kiron1/proxydetox/blob/main/pkg/macos/cc.colorto.proxydetox.plist "proxydetox launchd plist file"
