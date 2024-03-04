set -e

sh ./test.sh
cargo tauri build -- --bin sma-gui
mkdir -p ./target/prod-release-bundle
cp ./target/release/sma.exe ./target/release/sma-gui.exe ./target/release/bundle/msi/sma-gui* ./target/prod-release-bundle