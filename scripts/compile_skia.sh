#!/bin/bash
set -e

cd thirdparty/skia

python3 tools/git-sync-deps

bin/gn gen out/skbin

cat ../../scripts/skia_linux.gn | tee out/skbin/args.gn

ninja -C out/skbin
