---
description: Monitor and Manage Local AI Stack
---
// turbo-all

This workflow contains standard diagnostic and management commands for the Ollama and Open Notebook stack. By using `// turbo-all`, these commands are pre-approved for auto-run.

### Diagnostics
1. Check running containers:
`docker compose ps`

2. Monitor real-time resource usage:
`docker stats --no-stream`

3. List active Ollama models (via API, since `docker exec` is blocked by proxy):
`curl -s http://host.docker.internal:11434/api/tags`

### Logs
4. View Open Notebook logs:
`docker logs --tail 50 open_notebook`

5. View Ollama logs:
`docker logs --tail 50 ollama`

6. View SurrealDB logs:
`docker logs --tail 50 surrealdb`

### Management
7. Start the stack:
`docker compose up -d`

8. Stop the stack:
`docker compose stop`
