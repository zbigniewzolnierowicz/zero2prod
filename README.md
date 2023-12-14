# Zero to Production book project

## Configure linker

I used [mold](https://github.com/rui314/mold), simply install it with your package manager of choice and paste this into `~/.cargo/config.toml`

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=/usr/bin/mold"]
```

## Development loop

Install `cargo-watch` and run this command:

```bash
cargo watch -x check -x test -x run
```

[TODO]: use bacon instead

## Code coverage

Install `cargo-tarpaulin` and run this command:

```bash
cargo tarpaulin --ignore-tests
```
