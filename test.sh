set -e

mkdir -p "./gui/build"
(cd "./gui" && npm install)
cargo build --bin sma
cp ./target/debug/sma.exe ./gui/src-tauri/
cargo test --workspace