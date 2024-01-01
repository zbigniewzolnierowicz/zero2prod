# Zero to Production book project

## Configure linker

I used [mold](https://github.com/rui314/mold), simply install it with your package manager of choice and paste this into `~/.cargo/config.toml`

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=/usr/bin/mold"]
```

## Development loop

Install `bacon` and run this command:

```bash
bacon
```

## Code coverage

Install `cargo-tarpaulin` and run this command:

```bash
cargo tarpaulin --ignore-tests
```

## Better testing

Install `cargo-nextest` and run this command:

```bash
cargo nextest run
```

For continuous development testing, run:

```bash
bacon test
```

Or [run `bacon`](#development-loop) and press `T`.

## Environment variables to set

Get an API key from Postmark and set `APP_APPLICATION__EMAIL=<api key>`
