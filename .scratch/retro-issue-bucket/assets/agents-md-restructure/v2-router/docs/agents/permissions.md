# Permissions

Reach this doc when a task needs git, an install, a configuration change, or a permission change.

## Read the allowed permissions

Read `.pi/extensions/pi-permission-system/config.json` to see the allowed permissions.

Follow the permission system. Edit its config only with explicit user approval.

These restrictions exist to prevent the agent from touching git without supervision.

## Git

Commit only with explicit approval, even when the permission config allows commits.

Normal git commands are fine. Reserve destructive git commands for an explicit request.

## Permission changes

You may recommend a permission change at the end of a task. Apply it only with explicit user approval.
