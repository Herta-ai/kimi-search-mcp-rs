# 阶段 1：构建环境
# 使用基于 Alpine 的 Rust 镜像原生支持 musl
FROM rust:alpine AS builder

# 安装 musl 构建必要的工具
RUN apk add --no-cache musl-dev

# 创建空项目缓存依赖
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# 创建虚假的 main.rs 以缓存构建依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl

# 拷贝真实源码
COPY src ./src
# 更新时间戳以强制重新编译我们的代码
RUN touch src/main.rs
# 编译最终产物
RUN cargo build --release --target x86_64-unknown-linux-musl

# 阶段 2：极简运行环境
# gcr.io/distroless/static-debian12 是目前最小巧且安全的静态镜像（不到 3MB）
FROM gcr.io/distroless/static-debian12:latest

WORKDIR /app
# 从 builder 拷贝编译出的静态二进制文件
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/kimi-mcp-rust /app/kimi-mcp-rust

# 暴露端口
EXPOSE 3000

# 运行二进制文件
CMD ["/app/kimi-mcp-rust"]