# Package extra files

## proxydetox.service

Configuration file for `systemd(8)`, which can be placed in the users home
directory.

See the comments inside the file.

## com.github.kiron1.proxydetox.plist

Configuration for macOS `launchd(8)`.

- Copy the file into the directroy `~/Library/LaunchAgents`.
- `launchctl load -w -F ~/Library/LaunchAgents/com.github.kiron1.proxydetox.plist`
- `launchctl start com.github.kiron1.proxydetox`

