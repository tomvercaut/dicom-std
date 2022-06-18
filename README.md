# DICOM-std

A library and tools to parse the [DICOM](https://dicomstandard.org) standard and generate code based on it.

## Components

### Library

- [core](core) provides the model of the DICOM standard and the XML file format in which it is defined.
- [fetch](fetch) library to download the DICOM standard in XML format
- [test-data](test-data) library that generates test data by downloading parts of the DICOM standard in XML format
- [utils](utils) library to provide general utility functions shared within the project
- [parser/xml](parser/xml) provides a parser for the DICOM standard in XML format
- [apps/download](apps/download) application to download all or parts of the DICOM standard in XML format

## Building

You can use Cargo to build and test all crates in the repository.

```sh
cargo build
cargo test
```

## Roadmap & Contributing

The project is under active development and this is an initial set of development goals:

- [ ] Parse and model specific parts of the DICOM standard
    - [ ] composite IOD module tables
    - [ ] common IOD Modules tables
- [ ] Auto generate a library
    - [ ] model composite IOD modules and their attributes
    - [ ] implement a reader and writer for IOD modules

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.