set -e
cargo clean
(cd ./sma/testbins/test && cargo clean)
[ -e "./gui/src-tauri/sma.exe" ] && rm -v "./gui/src-tauri/sma.exe"
[ -e "./gui/build" ] && rm -vr ./gui/build
[ -e "./gui/node_modules" ] && rm -vr ./gui/node_modules
[ -e "./gui/.svelte-kit" ] && rm -vr ./gui/.svelte-kit
[ -e "./node_modules" ] && rm -vr ./node_modules