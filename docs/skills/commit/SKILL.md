---
name: commit
description: Create or revise Git commits in this project using English Conventional Commit messages and separate changes by context.
---

# Commit

- Inspect the staged and unstaged changes and group related changes into cohesive commits when the user requests commits. Preserve unrelated work.
- Write commit messages in English using Conventional Commits: `type(scope): subject` or `type: subject`. Use a suitable type such as `feat`, `fix`, `docs`, `refactor`, `test`, `build`, `ci`, `chore`, or `perf`; add a scope when it clarifies the context.
- Use a concise, lowercase, imperative subject without a final period. Follow any commitlint configuration present in the project. Include a body or breaking-change footer when needed to explain the change.
- Run checks appropriate to the changes and report any that were not run. Do not push or rewrite existing history unless the user authorizes it.
