# Package extra files

## proxydetox.service

Configuration file for `systemd(8)`, which can be placed in the users home
directory.

See the comments inside the file.

## cc.colorto.proxydetox.plist

Configuration for macOS `launchd(8)`.

- Copy the file into the directroy `~/Library/LaunchAgents`.
- `launchctl load -w -F ~/Library/LaunchAgents/cc.colorto.proxydetox.plist`
- `launchctl start cc.colorto.proxydetox`

