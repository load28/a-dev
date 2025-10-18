FROM node:20-slim

# 기본 도구 설치
RUN apt-get update && apt-get install -y \
    git \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Claude Code CLI 설치 (이미지에 포함하여 매번 설치 불필요)
RUN npm install -g @anthropic-ai/claude-code

# GitHub CLI 설치 (PR 생성용)
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg \
    && chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
    && apt-get update \
    && apt-get install -y gh \
    && rm -rf /var/lib/apt/lists/*

# Git 기본 설정
RUN git config --global user.name "AutoDev Bot" && \
    git config --global user.email "autodev@github-actions.bot"

# 작업 디렉토리
WORKDIR /workspace

# 출력 디렉토리
RUN mkdir -p /output

# 엔트리포인트 스크립트
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
