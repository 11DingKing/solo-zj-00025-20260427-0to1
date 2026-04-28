# 构建阶段
FROM --platform=$BUILDPLATFORM node:20-alpine as builder

WORKDIR /app

# 复制依赖文件
COPY package*.json ./

# 安装依赖
RUN npm install --legacy-peer-deps

# 复制源代码
COPY . .

# 构建生产版本
RUN npm run build -- --configuration production

# 运行阶段
FROM --platform=$TARGETPLATFORM nginx:alpine

# 复制构建产物
COPY --from=builder /app/dist/kanban-frontend/browser /usr/share/nginx/html

# 复制 nginx 配置
COPY nginx.conf /etc/nginx/conf.d/default.conf

# 暴露端口
EXPOSE 80

# 启动 nginx
CMD ["nginx", "-g", "daemon off;"]
