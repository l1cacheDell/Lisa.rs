<div style="width: 560px; height: 100px; overflow: hidden;">
    <img src="docs/bg.png" alt="描述" style="width: 100%; height: 100%; object-fit: cover;">
</div>

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