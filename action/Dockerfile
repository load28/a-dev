FROM python:3.11-slim

LABEL org.opencontainers.image.source="https://github.com/load28/a-dev"
LABEL org.opencontainers.image.description="AutoDev Task Executor - Execute coding tasks with Claude 4.5 Sonnet"
LABEL org.opencontainers.image.licenses="MIT"

WORKDIR /action

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy execution script
COPY execute_task.py .
RUN chmod +x execute_task.py

# Set GitHub workspace as working directory for task execution
WORKDIR /github/workspace

ENTRYPOINT ["python", "/action/execute_task.py"]
