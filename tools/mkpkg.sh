#!/bin/sh

set -eu

while getopts ":a:t:" o; do
  case "${o}" in
  a)
    arch=${OPTARG}
    ;;
  t)
    target=${OPTARG}
    ;;
  *)
    echo "fatal error: invalid arguments"
    exit 1
    ;;
  esac
done
shift $((OPTIND - 1))

: "${arch:=$(uname -m)}"

pkgid=cc.colorto.proxydetox
prefix=/opt/proxydetox
root=$(
  cd "$(dirname "$0")/.."
  pwd -P
)
workdir=$(mktemp -dt proxydetox-pkgbuild)
setproxy_helper=$(mktemp)

trap 'rm -rf "${workdir}" "${setproxy_helper}"' EXIT INT

plutil -lint "${root}/pkg/macos/${pkgid}.plist"
cargo install \
  --path "${root}/proxydetox" \
  --root "${workdir}/${prefix}" \
  --features negotiate \
  ${target:+--target ${target}} \
  --no-track
swiftc -o "${setproxy_helper}" "${root}/pkg/macos/setproxy.swift"
install -d "${workdir}/Library/LaunchAgents/"
install -v -m 0644 "${root}/pkg/macos/${pkgid}.plist" "${workdir}/Library/LaunchAgents/"
install -d "${workdir}/etc/paths.d/"
install -v -m 0644 "${root}/pkg/macos/40-proxydetox" "${workdir}/etc/paths.d/"
install -d "${workdir}/${prefix}/libexec/"
install -v "${setproxy_helper}" "${workdir}/${prefix}/libexec/setproxy_helper"

version=$(sed -n 's/^version[ \t]*=[ \t]*"\([0-9.]*\)"/\1/p' "${root}/proxydetox/Cargo.toml")
pkgfile=proxydetox-${version}-${arch}-apple-darwin.pkg
echo "version=${version}" >> "${GITHUB_OUTPUT:-/dev/stdout}"
echo "pkgfile=${pkgfile}" >> "${GITHUB_OUTPUT:-/dev/stdout}"

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
