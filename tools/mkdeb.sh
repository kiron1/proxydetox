#!/bin/sh

set -eu

features=
while getopts ":f:" o; do
  case "${o}" in
    f)
        features=${features:+${features},}${OPTARG}
        ;;
    *)
        echo "fatal error: invalid arguments"
        exit 1
        ;;
  esac
done
shift $((OPTIND-1))

prefix=/usr
root=$( cd "$(dirname "$0")/.." ; pwd -P )
workdir=$(mktemp -dt proxydetox-debbuild.XXXXXXXX)

trap "rm -rf ${workdir}" EXIT INT

mkdir -p "${workdir}/DEBIAN" "${workdir}/lib/systemd/user"

cargo install --path "${root}/proxydetox" --root "${workdir}/${prefix}" --no-track ${features:+--features=${features}}

sed -e "s/\${prefix}/${prefix//\//\\/}/" "${root}/debian/proxydetox.service" > "${workdir}/lib/systemd/user/proxydetox.service"

version=$(sed -n 's/^version\s*=\s*"\([0-9.]*\)"/\1/p' "${root}/proxydetox/Cargo.toml")
echo "::set-output name=version::${version}"

debfile=proxydetox-${version}-x86_64-linux.deb
echo "::set-output name=debfile::${debfile}"

sed -e "s/\${version}/${version}/" "${root}/debian/control" > "${workdir}/DEBIAN/control"
for f in postinst postrm; do
  cp "${root}/debian/${f}" "${workdir}/DEBIAN/${f}"
  chmod 0755 "${workdir}/DEBIAN/${f}"
done

echo Building ${debfile}
dpkg-deb --build "${workdir}" "${debfile}"

dpkg --info "${debfile}"
dpkg --contents "${debfile}"

echo
echo Install the package with:
echo sudo dpkg --install "${debfile}"
echo
