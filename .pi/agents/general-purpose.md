---
description: General-purpose agent for complex, multi-step implementation tasks
model: synthetic/hf:zai-org/GLM-5.2
permission:
  bash:
    "git *": deny
    "git status": allow
    "git status *": allow
    "git diff": allow
    "git diff *": allow
    "git log": allow
    "git log *": allow
    "git show": allow
    "git show *": allow
    "git blame": allow
    "git blame *": allow
    "git add": allow
    "git add *": allow
    "git fetch": allow
    "git fetch *": allow
    "git branch": allow
    "git branch *": allow
    "git remote": allow
    "git remote *": allow
    "git stash list": allow
    "git stash list *": allow
    "git branch -f *": deny
    "git branch --force *": deny
    "git clean -f *": deny
    "git clean --force *": deny
---
