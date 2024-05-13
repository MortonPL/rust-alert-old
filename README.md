# Rust Conquer

Structures and utilities for C&C formats written in Rust.

## TODO list

- Library
  - [x] CRC
  - [x] CSF
  - [x] INI
    - [ ] INIX
  - [x] MIX
    - [x] Core
    - [x] Blowfish/RSA
    - [x] LMD/GMD
    - [x] SHA1
  - [ ] PAL
  - [ ] SHP
- Tools
  - [x] CSF Builder
  - [x] MIX Multitool
  - [x] MIX DB Multitool
  - [x] MIX Cracker/Locker
  - [ ] INIX transpiler
  - [ ] Mod Builder
  - [ ] PAL Builder
- Other
  - [ ] Readme
    - [ ] Description
    - [ ] Shields
    - [ ] Installation
    - [ ] Usage
    - [ ] License
    - [ ] Third-party
    - [ ] Contributing/Development
  - [ ] Nice docs
  - [ ] CI
  - [ ] Profile and optimize stuff
  - [ ] Good tests/code coverage

### Running coverage

```sh
cargo tarpaulin --lib --out html --skip-clean
```
