# macOS

The prebuild Proxydetox packages for macOS come with the `proxydetoxctl` utility
shell script for an easier interaction with the launch agent of Proxydetox.

The Proxydetox daemon listens directly for the Kerberos SSO extension's
internal-network notifications. It switches the running proxy to direct mode
when the corporate network is unavailable and reloads the PAC file when the
network becomes available again.

See [Automatic macOS network switching](macos_network_switching.md) for the
runtime flow, package components, service lifecycle, and troubleshooting steps.

When called without any arguments it will show the status of the Proxydetox
launch agent.

The following sub-commands are supported:

- status
- start
- restart
- stop
- enable
- disable
