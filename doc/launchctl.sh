#!/bin/sh

install_and_enable() {
  # ANCHOR: install
  launchctl bootstrap "gui/$(id -u)" "/Library/LaunchAgents/cc.colorto.proxydetox.plist"
  launchctl enable "gui/$(id -u)/cc.colorto.proxydetox"
  launchctl print "gui/$(id -u)/cc.colorto.proxydetox"
  launchctl kickstart -kp "gui/$(id -u)/cc.colorto.proxydetox"
  # ANCHOR_END: install
}

disable_and_uninstall() {
  # ANCHOR: uninstall
  launchctl kill SIGTERM "gui/$(id -u)/cc.colorto.proxydetox"
  launchctl disable "gui/$(id -u)/cc.colorto.proxydetox"
  launchctl bootout "gui/$(id -u)/cc.colorto.proxydetox"
  # ANCHOR_END: uninstall
}

status() {
  # ANCHOR: print
  launchctl print "gui/$(id -u)/cc.colorto.proxydetox"
  # ANCHOR_END: print
}

case "${1:-status}" in
install)
  install_and_enable
  ;;
uninstall)
  disable_and_uninstall
  ;;
status)
  status
  ;;
*)
  echo "usage: ${0} install|uninstall"
  exit 1
  ;;
esac
