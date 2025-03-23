# Quick Start

```bash
cargo add rig-core  rig-sqlite
cargo add tokio --features macros,rt-multi-thread
```

结合sqlite3文档：https://github.com/0xPlaygrounds/rig/tree/main/rig-sqlite

debug看看back trace:

```bash
$env:RUST_BACKTRACE=1; cargo run
```