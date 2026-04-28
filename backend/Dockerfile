# 构建阶段
FROM --platform=$BUILDPLATFORM rust:1.88-alpine as builder

WORKDIR /app

# 安装构建依赖
RUN apk add --no-cache musl-dev openssl-dev

# 复制所有源代码
COPY Cargo.toml ./
COPY src ./src
COPY migrations ./migrations

# 直接编译
RUN cargo build --release

# 运行阶段
FROM --platform=$TARGETPLATFORM alpine:3.19

WORKDIR /app

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
