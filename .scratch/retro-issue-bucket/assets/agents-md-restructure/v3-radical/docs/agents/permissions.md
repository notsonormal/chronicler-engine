# Permissions

The permission system is configured in `.pi/extensions/pi-permission-system/config.json`. Read it to see what is allowed.

Do not circumvent the permissions it grants. You may *recommend* permission changes at the end of a task, but you may not *apply* them without explicit user approval — the restrictions exist to stop the agent from touching git without supervision.

Normal git commands are fine; avoid destructive git commands in general. Committing requires explicit approval (see the guardrails in `AGENTS.md`).
