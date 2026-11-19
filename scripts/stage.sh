#!/bin/sh
# stage.sh [armhf|armel|RUST_TARGET] [STAGE_DIR]
set -eu
cd "$(dirname "$0")/.."
case "${1:-armhf}" in
    armhf|armv7-unknown-linux-musleabihf) target=armv7-unknown-linux-musleabihf ;;
    armel|armv7-unknown-linux-musleabi) target=armv7-unknown-linux-musleabi ;;
    *) echo "Unsupported package architecture: $1" >&2; exit 1 ;;
esac
stage=${2:-build/stage}
target_dir=${CARGO_TARGET_DIR:-target}
bin=$target_dir/$target/release/habits-kindle
[ -x "$bin" ] || { echo "Build $bin before staging." >&2; exit 1; }
install -d "$stage/habits/bin" "$stage/habits/licenses" "$stage/documents"
install -m 755 "$bin" "$stage/habits/bin/habits-kindle"
install -m 755 deploy/Habits.sh "$stage/documents/Habits.sh"
install -m 644 licenses/chrono-LICENSE.txt "$stage/habits/licenses/chrono-LICENSE.txt"
