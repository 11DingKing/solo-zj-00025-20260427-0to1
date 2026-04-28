# 构建阶段
FROM --platform=$BUILDPLATFORM rust:1.88-alpine as builder

WORKDIR /app

# 使用国内 Alpine 镜像源加速 apk 下载
RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories

# 安装构建依赖
RUN apk add --no-cache musl-dev openssl-dev git ca-certificates

# 更新证书
RUN update-ca-certificates

# 配置 Cargo 镜像源 (使用阿里云)
RUN mkdir -p /root/.cargo && \
    cat > /root/.cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = "ustc"

[source.ustc]
registry = "git://mirrors.ustc.edu.cn/crates.io-index"

[net]
git-fetch-with-cli = true
retry = 10

[http]
timeout = 300
multiplexing = false
EOF

# 设置环境变量
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV CARGO_NET_RETRY=10
ENV CARGO_HTTP_TIMEOUT=300
ENV CARGO_HTTP_MULTIPLEXING=false

# 复制所有源代码
COPY Cargo.toml ./
COPY src ./src
COPY migrations ./migrations

# 直接编译
RUN cargo build --release

# 运行阶段
FROM --platform=$TARGETPLATFORM alpine:3.19

WORKDIR /app

# 使用国内 Alpine 镜像源加速 apk 下载
RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories

# 安装必要的运行时库
RUN apk add --no-cache ca-certificates libgcc

# 从构建阶段复制可执行文件
COPY --from=builder /app/target/release/kanban-backend ./kanban-backend

# 复制迁移文件
COPY --from=builder /app/migrations ./migrations

# 暴露端口
EXPOSE 8080

# 启动命令
CMD ["./kanban-backend"]
