#!/bin/sh

set -eu

pkgid=com.github.kiron1.proxydetox
prefix=/usr/local
root=$( cd "$(dirname "$0")/.." ; pwd -P )
workdir=$(mktemp -dt proxydetox-pkgbuild)

trap "rm -rf ${workdir}" EXIT ERR


cargo install --path "${root}/proxydetox" --root "${workdir}" --no-track --features gssapi
version=$(sed -n 's/^version[ \t]*=[ \t]*"\([0-9.]*\)"/\1/p' "${root}/proxydetox/Cargo.toml")
echo "::set-output name=version::${version}"

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
