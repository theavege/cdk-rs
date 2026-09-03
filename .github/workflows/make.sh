#!/usr/bin/env bash

set -euo pipefail

cdk_config="$(command -v cdk5-config || find /usr/lib -type f -name cdk5-config -print -quit)"
if [[ -z "${cdk_config}" ]]; then
    printf 'cdk5-config is required; install the CDK development package.\n' >&2
    exit 1
fi
cdk_bin_dir="$(dirname "${cdk_config}")"
export PATH="${cdk_bin_dir}:${PATH}"

shellcheck --external-sources "${0}"
shfmt -ci -fn -i 4 -d "${0}"

cargo clippy --quiet --examples
cargo test --workspace --all-targets
cargo build --release --examples
cargo fmt --check --all
