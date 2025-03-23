<img src="docs/bg.png" alt="background" style="width: 800px; height: 150px; overflow: hidden;object-fit: cover;">

# Quick Start

```bash
git clone https://github.com/l1cacheDell/Lisa.rs.git
cd Lisa.rs

# build this project
cargo build

# run this project
cargo run
```

# 框架技术栈

+ 向量数据库方案：sqlite3：https://github.com/0xPlaygrounds/rig/tree/main/rig-sqlite
+ Agent框架：https://docs.rig.rs/guides
+ 后端微服务框架：https://actix.rs/docs/getting-started/

# utils
如果需要debug看看back trace:

```bash
$env:RUST_BACKTRACE=1; cargo run
```