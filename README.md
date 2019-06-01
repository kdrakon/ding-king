# Ding-King
aka _"Bell"-"Roy"_

## Usage

```bash
ding-king

USAGE:
    ding-king --email <EMAIL> --name <NAME> --url <POST_URL>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --email <EMAIL>
        --name <NAME>
        --url <POST_URL>
```

### Example
```bash
cat input | ding-king --email foo@foo.com --name Bar --url https://post-url.com
```

### Input
See [input file](input) for example.

## Build
1. Install [rustup](https://rustup.rs/), which will install `rustc` and `cargo`
1. Run `cargo build --release`
1. The binary for your platform will be at `target/release/ding-king`.