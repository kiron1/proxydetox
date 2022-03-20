# Proxydetox as user service

Register Proxydetox as a user service such that it is automatically started by
the system when the user is logged in.

## Automatically start Proxydetox with a user session

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
