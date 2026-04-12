---
trigger: always_on
---

# Environment & Security Rules

## Devcontainer Environment
This project is configured to run inside a **Linux-based devcontainer**. 

## Docker Access
- Docker commands are routed through a **restricted docker-proxy** at `host.docker.internal:2375`
- The proxy limits access to only `CONTAINERS`, `IMAGES`, and `NETWORKS` operations (no exec/post)
- Direct access to `/var/run/docker.sock` is not available
- This allows safe management of containers from within the devcontainer

## Access Restrictions
- **Workspace Scoping**: Agents and users are restricted to the `/workspaces` directory. Access to the host system is not permitted.
- **Privileges**: The current shell does not have root access. Attempts to read sensitive system files (e.g., `/etc/shadow`) or access `/root` will result in `Permission denied`.

## Path Information
- Use container-local absolute paths when referencing files.
- Host paths (e.g., `D:/...` on Windows) are mapped to `/workspaces/...` mount points.