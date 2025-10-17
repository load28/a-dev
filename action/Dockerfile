FROM node:20-alpine

# Install system dependencies
RUN apk add --no-cache \
    bash \
    git \
    curl \
    github-cli

# Install Claude Code CLI
RUN npm install -g @anthropic-ai/claude-code

# Copy entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Set working directory
WORKDIR /workspace

ENTRYPOINT ["/entrypoint.sh"]
