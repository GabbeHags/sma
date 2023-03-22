# Start Multiple Applications (SMA)

### Info
This project is only tested on Windows.

### TODO
Add `--json` to the arguments, this should make it possible to use the keys from a json file in the `--start` which should make it possible to get shorter application names, and you don't need to specify the args to the application.

### How to use
#### USAGE:
    sma.exe [OPTIONS]

#### OPTIONS:
        --exit <exit>         Exits all the started applications if the application name given to
                              `--exit` is exited.
    -h, --help                Print help information
        --json <json>         Uses a json file for the `--start` and `--exit` instead of the command
                              line arguments.
        --start <start>...    Starts the given applications.


### How to build
`$ cargo build`

The tests use the executables in the test_program folder, if you want to recompile the test executables run the `$ build_test.sh` in the test_programs folder.

### Examples

`$ sma.exe --start "test_program/test.exe 2" "test_program/test2.exe 10" --exit test.exe`

The line above will start 2 applications `test.exe` and `test2.exe` where they will sleep for 2 and 10 sec respectively. But because we added `--exit test.exe` the second application (`test2.exe`) will be killed after 2 sec because the first applications (`test.exe`) exited.

### Versions

* cargo 1.68.0
* rustc 1.68.0

### License
This project is released under the [MIT License](LICENSE)