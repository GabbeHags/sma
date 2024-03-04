mkdir -p "./gui/build"
(cd "./gui" && npm install)
cargo tauri dev -- --bin sma-gui
