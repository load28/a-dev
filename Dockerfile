FROM rust:1.75-slim as builder

WORKDIR /app

# 의존성 캐싱을 위한 더미 프로젝트
RUN USER=root cargo new --bin autodev-clone
WORKDIR /app/autodev-clone

# Cargo 파일 복사 및 의존성 빌드
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
RUN rm src/*.rs

# 실제 소스 코드 복사 및 빌드
COPY src ./src
RUN rm ./target/release/deps/autodev*
RUN cargo build --release

# 런타임 이미지
FROM debian:bookworm-slim

# 필요한 런타임 라이브러리 설치
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 빌드된 바이너리 복사
COPY --from=builder /app/autodev-clone/target/release/autodev /app/autodev

# 설정 파일 복사
COPY config.toml /app/config.toml

# 포트 노출
EXPOSE 3000

# 헬스체크
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

ENTRYPOINT ["/app/autodev"]
CMD ["serve", "--port", "3000"]