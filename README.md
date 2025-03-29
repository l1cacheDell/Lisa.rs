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

```
I used to believe in love the way everyone does when they are young and naive as a perfect unbreakable bond that ties two people together I never thought it could fade not even a little But over the years I've come to understand that love isn't always forever Sometimes it’s just a fleeting moment in time and when that moment passes the truth we leave behind is all that remains I remember the first time I met you It was one of those encounters that felt like destiny as if the universe had conspired for our paths to cross You had that warmth in your eyes that smile that could make even the darkest of days feel bright You were my everything my best friend my lover my confidante I thought we would grow old together hand in hand through every storm and sunny day But as the years went by things began to shift It wasn’t dramatic at first It was subtle like the quiet rustling of leaves in the wind that you barely notice until one day you realize that the trees are bare Our conversations became shorter our moments of silence longer The affection we once shared began to feel like a distant memory and I couldn't help but wonder when did it all start to change At first I told myself it was just a phase Life gets in the way sometimes work stress the everyday chaos But deep down I knew it wasn’t just that It was something deeper Something neither of us were willing to face I remember the night you left It wasn't a dramatic scene there were no shouting matches no tear-filled goodbyes It was just you standing there quietly packing your things preparing to walk away You didn’t say much but I could feel the weight of your silence It hurt more than any angry words could have And as you walked out of that door I realized something love isn’t always about staying Sometimes it's about knowing when to leave The truth is love fades People grow change and sometimes they simply drift apart It’s not anyone’s fault it’s just the way things are The love we once had wasn’t enough to keep us together and neither of us had the strength to fight for it anymore But even though you left I don’t hold any resentment I’ve learned to accept the truth that sometimes people are meant to walk out of our lives and we must learn to let them go The truth that you left is a hard one to swallow but it’s a truth that has taught me more about myself than I ever expected
```