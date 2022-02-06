[![Rust](https://github.com/GabbeHags/sma/actions/workflows/rust.yml/badge.svg)](https://github.com/GabbeHags/sma/actions/workflows/rust.yml)

# Start Multiple Applications (SMA)

### Info
This project is only tested on Windows.

### TODO
Add ``--json`` to the arguments, this should make it possible to use the keys from a json file in the ``--start`` which should make it possible to get shorter application names, and you don't need to specify the args to the application.

### How to use
``--start``, Starts the given applications.

``--exit``, Exits all the started applications if the argument given to --exit is exited.

### How to build
``$ cargo build``

### Versions

* cargo 1.58.0
* rustc 1.58.1