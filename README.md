<div style="width: 560px; height: 100px; overflow: hidden;">
    <img src="docs/bg.png" alt="描述" style="width: 100%; height: 100%; object-fit: cover;">
</div>

# 1. Quick Start

## 1.1 Prepare your ENVIRON vars
在启动本项目之前，需要先创建一份`.env`文件，并且填写以下字段：

```bash
BASE_URL="your_model_service_base_url"
MODEL_NAME="your_model_name"
EMBEDDING_MODEL_NAME="the_name_of_embedding_model"
EMBEDDING_MODEL_NDIM="the_ndim_of_embedding_model"
OPENAI_API_KEY="your_api_key"
PORT="8080"
DB_PATH="data/vector_store.db"      # 这个建议default, 就用这个路径文件，不用改。
```

## 1.2 Run executable from source
在填写好`.env`文件之后，就可以编译、启动本项目：

```bash
git clone https://github.com/l1cacheDell/Lisa.rs.git
cd Lisa.rs

# build this project
cargo build

# run this project
cargo run
```

## 1.3 Run executable from Github Release

或者，如果没有办法编译，可以直接从release里面下载预先编译好的文件，直接启动即可。

Windows:

```bash
# 先从github下载可执行文件
...

# 注意：请一定与.env在同一路径下启动！
lisa.exe
```

Linux:

```bash
# 先通过wget从github获取二进制文件
...

# 注意：请一定与.env在同一路径下启动！
./lisa
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

# Debug记录
Rig这个Tool，做出来只会直接返回结果。需要探究一下到底怎么才能不直接返回结果。

看了一下源码，Tool应该就是单纯的tool，跟python那边不一样。

如果要做RAG功能，那就是直接走dynamic_context()函数；

如果要做**基于RAG的其他任务**，那就是dynamic_tools(sample: usize, index, Toolset)函数。