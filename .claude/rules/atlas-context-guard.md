---
paths:
  - "crates/**"
  - "**/*.rs"
---

# Context Guard Protocol

Auto-loaded when editing Rust source files.

When you see `[CTX N%]` warnings from the PostToolUse hook:

- **79-86%**: Mentally identify what remains. No new large tasks inline.
- **87-92%**: Stop new implementation. Finish current edit cleanly, then delegate remaining work to a Sonnet agent with a structured handoff: what's done, what's left, which files, acceptance criteria.
- **93%+**: Immediately stop. Write a handoff to `/tmp/atlas-handoff-{session_id}.md` with: progress summary, remaining tasks, key decisions made, files touched. Delegate to Sonnet agent with that file as context. Do NOT continue inline work.

**Handoff must include:** (1) what was accomplished (2) what remains with file paths (3) decisions/context the delegate needs (4) how to verify the work.

**Sub-agents ignore this hook.** Only the orchestrator session receives warnings.
