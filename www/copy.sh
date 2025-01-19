#!/usr/bin/env bash

set -euo pipefail

# When not arguemnt is given, copy to the workspace root directory.
dst=${1:-${BUILD_WORKSPACE_DIRECTORY}/public}

# Take care of relative path, since this scripts working directoy is
# not the one where `bazel run ...` was called.
case "${dst}" in
  /*) ;;
  *) dst=${BUILD_WORKING_DIRECTORY}/${dst}
esac

mkdir -p "${dst}"
tar -xzvf "${WWW_TARBALL}" -C "${dst}"
