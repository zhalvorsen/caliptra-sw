#!/bin/bash
set -ex

CPTRA_UIO_NUM=4 cargo nextest archive --features=fpga_realtime,itrng --archive-file nextest-archive/test.tar.zst
(
    cd nextest-archive
    git add test.tar.zst
    git ammend
    git push -f
)

git add nextest-archive
git ammend
git push -f
