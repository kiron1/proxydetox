#!/bin/sh

set -eu

pkgid=cc.colorto.proxydetox
prefix=/opt/proxydetox
root=$(
  cd "$(dirname "$0")/.."
  pwd -P
)
workdir=$(mktemp -dt proxydetox-pkgbuild)

trap 'rm -rf ${workdir}' EXIT INT

plutil -lint "${root}/pkg/macos/${pkgid}.plist"
cargo install \
  --path "${root}/proxydetox" \
  --root "${workdir}/${prefix}" \
  --features negotiate \
  --no-track
install -d "${workdir}/Library/LaunchAgents/"
install -v -m 0644 "${root}/pkg/macos/${pkgid}.plist" "${workdir}/Library/LaunchAgents/"
install -d "${workdir}/etc/paths.d/"
install -v -m 0644 "${root}/pkg/macos/40-proxydetox" "${workdir}/etc/paths.d/"

version=$(sed -n 's/^version[ \t]*=[ \t]*"\([0-9.]*\)"/\1/p' "${root}/proxydetox/Cargo.toml")
pkgfile=proxydetox-${version}-x86_64-apple-darwin.pkg
echo "::set-output name=version::${version}"
echo "::set-output name=pkgfile::${pkgfile}"

echo "Building ${pkgfile}"
pkgbuild \
  --root "${workdir}" \
  --install-location "/" \
  --identifier "${pkgid}" \
  --version "${version}" \
  --scripts "${root}/pkg/macos/scripts" \
  --ownership recommended \
  "${pkgfile}"

#lsbom $(pkgutil --bom "${pkgfile}")

echo
echo Install the package with:
echo sudo installer -package "${pkgfile}" -target /
echo
