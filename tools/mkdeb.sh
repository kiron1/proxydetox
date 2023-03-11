#!/bin/sh

set -eu

distname=linux
features=
no_default_features=
while getopts ":d:f:n" o; do
  case "${o}" in
  d)
    distname=${OPTARG}
    ;;
  f)
    features=${features:+${features},}${OPTARG}
    ;;
  n)
    no_default_features=y
    ;;
  *)
    echo "fatal error: invalid arguments"
    exit 1
    ;;
  esac
done
shift $((OPTIND - 1))

prefix=/usr
root=$(
  cd "$(dirname "$0")/.."
  pwd -P
)
workdir=$(mktemp -dt proxydetox-debbuild.XXXXXXXX)

trap 'rm -rf ${workdir}' EXIT INT

mkdir -p "${workdir}/DEBIAN" "${workdir}/lib/systemd/user"

cargo install \
  --path "${root}/proxydetox" \
  --root "${workdir}/${prefix}" \
  --no-track \
  ${no_default_features:+--no-default-features} \
  ${features:+--features=${features}}

sed -e "s|\${prefix}|${prefix}|" "${root}/debian/proxydetox.service" \
  >"${workdir}/lib/systemd/user/proxydetox.service"

version=$(sed -n 's/^version\s*=\s*"\([0-9.]*\)"/\1/p' "${root}/proxydetox/Cargo.toml")
echo "version=${version}" >> "${GITHUB_OUTPUT:-/dev/stdout}"

pkgfile=proxydetox-${version}-x86_64-${distname}.deb
echo "pkgfile=${pkgfile}" >> "${GITHUB_OUTPUT:-/dev/stdout}"

sed -e "s/\${version}/${version}/" "${root}/debian/control" >"${workdir}/DEBIAN/control"
for f in postinst postrm; do
  cp "${root}/debian/${f}" "${workdir}/DEBIAN/${f}"
  chmod 0755 "${workdir}/DEBIAN/${f}"
done

echo "Building ${pkgfile}"
dpkg-deb --build "${workdir}" "${pkgfile}"

dpkg --info "${pkgfile}"
dpkg --contents "${pkgfile}"

echo
echo Install the package with:
echo sudo dpkg --install "${pkgfile}"
echo
