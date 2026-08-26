#!/usr/bin/env bash

set -euo pipefail

if ! command -v cdk5-config >/dev/null; then
    source '/etc/os-release'
    case ${ID:?} in
        debian | ubuntu) sudo bash -c '
            apt-get update
            apt-get install -y shfmt cppcheck shellcheck libcdk5-dev
        ' ;;
        fedora | alma) sudo dnf install -y shfmt cppcheck shellcheck cdk-devel ;;
    esac 1>/dev/null
fi

shellcheck --external-sources "${0}"
shfmt -ci -fn -i 4 -d "${0}"

cargo clippy --quiet --examples
cargo build --release --examples
cargo fmt --check --all
