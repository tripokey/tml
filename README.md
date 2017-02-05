# tml

[![Build Status](https://travis-ci.org/tripokey/tml.svg?branch=master)](https://travis-ci.org/tripokey/tml)

```
tml creates a symbolic link to SOURCE at DESTINATION, creating parent directories as needed.
DESTINATION defaults to the basename of SOURCE if omitted.
The basename of SOURCE will be appended to DESTINATION if DESTINATION ends with a '/'.

USAGE:
    tml [FLAGS] <SOURCE> [ARGS]

FLAGS:
    -n               do not verify that SOURCE exists
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <SOURCE>         the target of the link
    <DESTINATION>    The destination path to create
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
