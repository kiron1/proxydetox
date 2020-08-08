#!/bin/sh

set -eu

pkgid=com.github.kiron1.proxydetox
prefix=/usr/local
root=$( cd "$(dirname "$0")/.." ; pwd -P )
workdir=$(mktemp -dt proxydetox-pkgbuild)

trap "rm -rf ${workdir}" EXIT ERR


cargo install --path "${root}" --root "${workdir}" --no-track
version=$("${workdir}/bin/proxydetox" --version | sed -n 's/proxydetox \([0-9.]*\)/\1/p')

pkgfile=proxydetox-${version}-x86_64-apple-darwin.pkg
echo "::set-output name=pkgfile::${pkgfile}"

echo Building ${pkgfile}
pkgbuild \
	--root "${workdir}" \
	--install-location "${prefix}"\
	--scripts "${root}/pkg/macos" \
	--identifier "${pkgid}" \
	--version ${version} \
	--ownership recommended \
	"${pkgfile}"

#lsbom $(pkgutil --bom "${pkgfile}")

echo
echo Install the package with:
echo sudo installer -package "${pkgfile}" -target /
echo
