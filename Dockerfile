FROM rust:latest
WORKDIR /app
RUN apt update && apt install mold clang -y

COPY . .

# 告诉 SQLx 开启离线模式，直接读取 .sqlx 文件夹
ENV SQLX_OFFLINE=true

RUN cargo build --release

# ⚠️ 注意：请把这里的 zero2prod 换成你 Cargo.toml 里的 [package] name！
# 比如我看你的 GitHub 叫 web_render，如果你的程序名是它，就改成 ./target/release/web_render
ENTRYPOINT ["./target/release/myzero2prod"]