# Atlas Communications Standard

Applies to: PR titles, PR bodies, commit messages, code comments, docs, issue text — anything public-facing.

## Tone

**Professional and factual.** Describe what changed and why. No superlatives, no hype, no internal framing.

## Banned phrases

- "world-class", "best-in-class", "cutting-edge", "next-generation"
- "powerful", "robust", "blazing fast" (unless citing a benchmark)
- "revolutionary", "innovative", "state-of-the-art"
- Any internal framing: "AI-driven development", "AI-first standards", "100% AI developed"
- Grandiose scope claims: "overhaul", "revolutionize", "transform" — use "update", "fix", "add", "change"

## Commit messages

Follow conventional commits. Title is imperative, factual, specific.

```
✅ ci: add macOS to post-merge test matrix
✅ fix: correct path separator handling on Windows
✅ feat: add vm_fuzz target to nightly schedule

❌ ci: overhaul workflows for world-class compiler standards
❌ feat: revolutionary new type inference engine
```

## PR titles

Same rules as commit messages. Concise, describes the change.

## PR bodies

- **What** — what changed (factual)
- **Why** — reason for the change (technical justification)
- No marketing language, no internal motivations, no AI attribution in the body text

## Code comments

Explain the *why* when non-obvious. No praise, no hype.

## The rule of thumb

If it reads like a press release, rewrite it. If it reads like a changelog entry, it's correct.
