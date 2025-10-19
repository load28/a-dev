FROM node:20-slim

LABEL org.opencontainers.image.source="https://github.com/load28/a-dev"
LABEL org.opencontainers.image.description="AutoDev Claude Code Executor - Local Docker execution environment"
LABEL org.opencontainers.image.licenses="MIT"

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    ca-certificates \
    gnupg \
    && rm -rf /var/lib/apt/lists/*

# Install GitHub CLI
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | \
    dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg && \
    chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | \
    tee /etc/apt/sources.list.d/github-cli.list > /dev/null && \
    apt-get update && \
    apt-get install -y gh && \
    rm -rf /var/lib/apt/lists/*

# Install Claude Code CLI
RUN npm install -g @anthropic-ai/claude-code

# Configure Git
RUN git config --global user.name "AutoDev Bot" && \
    git config --global user.email "autodev@github-actions.bot" && \
    git config --global init.defaultBranch main

# Set working directory
WORKDIR /workspace

# Default command (can be overridden)
CMD ["bash"]
