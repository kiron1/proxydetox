#!/bin/sh

set -eu

prefix=/usr
root=$( cd "$(dirname "$0")/.." ; pwd -P )
workdir=$(mktemp -dt proxydetox-debbuild.XXXXXXXX)

trap "rm -rf ${workdir}" EXIT INT

mkdir -p "${workdir}/usr/bin"
if [ -n "${1:-}" ]; then
  cp "${1}" "${workdir}/usr/bin"
  strip "${workdir}/usr/bin/$(basename ${1})"
else
  cargo install --path "${root}" --root "${workdir}/usr" --no-track
fi

mkdir -p "${workdir}/usr/lib/systemd/user"
cat > "${workdir}/usr/lib/systemd/user/proxydetox.service" <<EOF
[Unit]
Description=Proxydetox Daemon

[Service]
ExecStart=/usr/bin/proxydetox %h/.config/proxydetox/proxy.pac 3128

[Install]
WantedBy=default.target
EOF

version=$(sed -n 's/^version\s*=\s*"\([0-9.]*\)"/\1/p' "${root}/Cargo.toml")
echo "::set-output name=version::${version}"

debfile=proxydetox-${version}-x86_64-linux.deb
echo "::set-output name=debfile::${debfile}"

mkdir -p "${workdir}/DEBIAN"

cat > "${workdir}/DEBIAN/control" <<EOF
Package: proxydetox
Version: ${version}-1
Section: base
Priority: optional
Architecture: amd64
Maintainer: Kiron <kiron1@gmail.com>
Description: Proxydetox
 A proxy for localhost to simplify life with corperate proxies.

EOF

echo Building ${debfile}
dpkg-deb --build "${workdir}" "${debfile}"

dpkg --info "${debfile}"
dpkg --contents "${debfile}"

echo
echo Install the package with:
echo sudo dpkg --install "${debfile}"
echo
