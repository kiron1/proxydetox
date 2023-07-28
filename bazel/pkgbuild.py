#!/usr/bin/env python3

import argparse
import os
import subprocess
import sys
import tempfile
import tarfile


def args():
    parser = argparse.ArgumentParser(prog="pkgbuild", fromfile_prefix_chars="@")
    parser.add_argument("--out", action="store", type=str, required=True)
    parser.add_argument(
        "--data", action="store", type=argparse.FileType(mode="rb"), required=True
    )
    parser.add_argument("--identifier", action="store", type=str, required=True)
    parser.add_argument("--install-location", action="store", type=str, required=True)
    parser.add_argument(
        "--postinstall", action="store", type=argparse.FileType(mode="r"), default=None
    )
    parser.add_argument(
        "--preinstall", action="store", type=argparse.FileType(mode="r"), default=None
    )
    parser.add_argument("--version", action="store", type=str, required=True)
    return parser.parse_args()


def pkgbuild(*, out, root_dir, scripts_dir, install_location, identifier, version):
    if sys.platform == "darwin":
        cmd = [
            "pkgbuild",
            "--root",
            root_dir,
            "--install-location",
            install_location,
            "--identifier",
            identifier,
            "--version",
            version,
            "--scripts",
            scripts_dir,
            "--ownership",
            "recommended",
            out,
        ]
    else:
        wd = os.path.commonpath([root_dir, scripts_dir])
        cmd = [
            "tar",
            "cf",
            out,
            "-C",
            wd,
            os.path.basename(root_dir),
            os.path.basename(scripts_dir),
        ]
    subprocess.run(cmd, check=True)


def main(args):
    try:
        os.remove(args.out)
    except FileNotFoundError:
        pass

    with tempfile.TemporaryDirectory(
        prefix="_pkbbuild_root_", dir=os.getcwd()
    ) as tmpdir:
        root_dir = os.path.join(tmpdir, "root")
        os.makedirs(root_dir)

        scripts_dir = os.path.join(tmpdir, "scripts")
        os.makedirs(scripts_dir)

        with tarfile.open(fileobj=args.data) as tar:
            tar.extractall(path=root_dir)

        if args.postinstall:
            with open(os.path.join(scripts_dir, "postinstall"), "w") as f:
                f.write(args.postinstall.read())

        if args.preinstall:
            with open(os.path.join(scripts_dir, "preinstall"), "w") as f:
                f.write(args.preinstall.read())

        pkgbuild(
            out=args.out,
            root_dir=root_dir,
            scripts_dir=scripts_dir,
            install_location=args.install_location,
            identifier=args.identifier,
            version=args.version,
        )


if __name__ == "__main__":
    main(args())
