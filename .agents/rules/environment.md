---
trigger: always_on
---

# Environment & Security Rules

## Devcontainer Environment
This project is configured to run inside a **Linux-based devcontainer**. 


## Access Restrictions
- **Workspace Scoping**: Agents and users are restricted to the `/workspaces` directory. Access to the host system is not permitted.
- **Privileges**: The current shell does not have root access. Attempts to read sensitive system files (e.g., `/etc/shadow`) or access `/root` will result in `Permission denied`.
- **System Resources**: Host-level sockets (like `docker.sock`) are not available, ensuring strong container isolation.

## Path Information
- Use container-local absolute paths when referencing files.
- Note that host paths (e.g., `C:\...`) are mapped to `/workspaces/...` mount points.