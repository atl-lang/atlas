
# Atlas Communications Standard

Applies to: PR titles, PR bodies, commit messages, code comments, docs, changelogs — anything written on behalf of Atlas.

## Core principle

Public-facing text describes **what changed and why**, technically and accurately. It does not editorialize, sell, or project ambitions onto the work.

## Tone

- Factual and precise
- GitHub-standard professional (same register as rust-lang, tokio, ripgrep PRs)
- Atlas identity language is fine in appropriate contexts (docs, README, marketing copy):
  - "AI-first", "AI-driven", "100% AI developed" — these are Atlas canon, use them where relevant
- Aspirational claims about the project's future do not belong in PRs, commits, or changelogs

## What doesn't belong in PRs and commits

- Superlatives describing quality: "world-class", "best-in-class", "blazing fast" (unless citing a benchmark)
- Scope framing: "overhaul", "revolutionize", "transform" — prefer "update", "fix", "add", "change", "remove"
- Motivational narrative: explaining *why we care* rather than *what changed*
- Audience awareness: PRs are read by contributors, not customers

## Commit messages

Conventional commits format. Imperative, specific, no filler.

```
# Good
ci: update MSRV from 1.70 to 1.85
ci: add Windows to PR test matrix, move macOS to post-merge
fix: correct path separator handling on Windows
feat: add vm_fuzz target to nightly schedule

# Bad
ci: overhaul workflows for world-class compiler standards
feat: revolutionary new type inference engine
fix: make the compiler not crash (super important!!!)
```

## PR titles

One line. Describes the change, not the motivation. Same rules as commit messages.

## PR bodies

Standard sections:
- **What** — factual list of what changed
- **Why** — technical justification (not a vision statement)

If the why is obvious from the what, omit it. No closing remarks, no sign-offs beyond the standard co-author trailer.

## The test

Read it back. If it could appear unchanged in a changelog entry for a mature open-source project — it's correct. If it sounds like it's trying to impress someone — rewrite it.
