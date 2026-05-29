# 修改这里：指定使用 bookworm 版本的 cargo-chef，确保与运行阶段的 GLIBC 版本严格一致
FROM lukemathwalker/cargo-chef:latest-rust-bookworm AS chef
WORKDIR /app

RUN apt update && apt install mold clang -y
# 阶段 2: 提炼依赖 Planner
FROM chef AS planner
COPY . .
# 剥离源码，只提取 Cargo.toml 和 Cargo.lock 生成 recipe.json
RUN cargo chef prepare --recipe-path recipe.json

# 阶段 3: 真正的构建工坊 Builder
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# 先把几百个第三方依赖编译并缓存起来
# 只要 Cargo.toml 没改动，下一次构建时这一步直接 0 秒跳过
RUN cargo chef cook --release --recipe-path recipe.json

# 现在把源码复制进来
COPY . .
ENV SQLX_OFFLINE=true

# 真正编译你的项目。因为依赖项已经全部缓存，这一步只需要几秒钟
RUN cargo build --release

# 使用完全一致的 debian:bookworm-slim 瘦身版作为运行环境
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# 安装线上运行时必要的底层安全证书和基础包
RUN apt-get update -y && apt-get install -y --no-install-recommends \
    openssl \
    ca-certificates \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/myzero2prod myzero2prod
COPY configuration configuration

ENV APP_ENVIRONMENT=production

ENTRYPOINT ["./myzero2prod"]