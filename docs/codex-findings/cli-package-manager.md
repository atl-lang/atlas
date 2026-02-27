# CLI Package Manager Stubs

Target: atlas-cli
Severity: Medium
Status: Open

## Finding 1: `atlas install` simulates registry downloads

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-cli/src/commands/install.rs:152-184

What/Why:
- The install command creates placeholder directories and files instead of downloading from a registry.

Impact:
- Dependency resolution in real projects will appear to succeed but does not produce actual package contents.

Recommendation:
- Integrate registry client or at least error when registry access is unavailable, to avoid false success.

---

## Finding 2: `atlas publish` does not upload, test, or package

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-cli/src/commands/publish.rs:177-190
- /Users/proxikal/dev/projects/atlas/crates/atlas-cli/src/commands/publish.rs:307-316
- /Users/proxikal/dev/projects/atlas/crates/atlas-cli/src/commands/publish.rs:318-345

What/Why:
- Registry upload is simulated, tests are skipped, and tarball creation writes a placeholder file.

Impact:
- Publishing flow is not trustworthy; it can report success without producing a valid package artifact.

Recommendation:
- Implement actual test execution (`atlas test`), archive creation, and registry upload; fail hard if any step is missing.
