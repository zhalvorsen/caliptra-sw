#!/bin/bash
set -ex

git fetch --all
git checkout zhalvorsen/test_sign
git submodule update --init

CPTRA_UIO_NUM=4 cargo nextest run --no-fail-fast --features=fpga_realtime,itrng -E 'test(test_invoke_dpe_sign_and_certify_key_cmds)'
