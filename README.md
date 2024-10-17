# zstd-sfx

zstd-sfx is a project to compress data into a self-extracting executable with [zstd](https://github.com/facebook/zstd) compression algorithm. This project is composed of two components: `archiver` and `unarchiver`, the `archiver` would compress the data and combine with the `unarchiver`, into a single executable.

## Usage

The usage of the `archiver` is shown below

```
Usage: archiver <target> <unarchiver> [-l <level>] [-o <output>] [-r] [-t <temp>]

Positional Arguments:
  target            what to compress
  unarchiver        path to unarchiver executable

Options:
  -l, --level       zstd compress level, range from 0~23, default is 3
  -o, --output      path to output sfx file, default is target name
  -r, --root        the root directory with be included or not
  -t, --temp        where temp files should be store, default depends on os
  --help            display usage information
```

To use zstd-sfx, you can download the [pre-compiled binaries here](https://github.com/phoxwupsh/zstd-sfx/releases). Or you can compile by yourself, make sure you have [rust toolchain](https://www.rust-lang.org/tools/install) and [git](https://git-scm.com/) installed first, you'll need to download the source code with git

```shell
git clone https://github.com/phoxwupsh/zstd-sfx.git
```

Then build `unarchiver` first

```shell
cd zstd-sfx
cargo build --bin=unarchiver --release
```

After you build the `unarchiver`, the executable file would be `target/release/unarchiver` in the project directory (for Windows is `/target/release/unarchiver.exe`)

So you can compress the data like this

**Unix**

```shell
cargo run --bin=archiver --release -- some/path/to/data target/release/unarchiver -o sfx-executable
```

**Windows**
```shell
cargo run --bin=archiver --release -- some/path/to/data target/release/unarchiver.exe -o sfx-executable.exe
```

## About binary size

If the executable file size of `unarchiver` is a concern for you, you can do some trick to shrink it.

First you would need rust nightly

```shell
rustup install nightly
```

Then you can compile it with

**Unix**
```shell
# Find your host's target
rustc -vV
RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-gnu --bin=unarchiver --release
```

**Windows**
```shell
$env:RUSTFLAG="-Zlocation-detail=none -Zfmt-debug=none"
cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-pc-windows-msvc --bin=unarchiver --release
```

For more information, refer to [johnthagen/min-sized-rust](https://github.com/johnthagen/min-sized-rust)