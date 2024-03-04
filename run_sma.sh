mkdir -p "./gui/build"
(cd "./gui" && npm install)
cargo run --bin sma -- "$@"