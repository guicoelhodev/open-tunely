---
name: code-review
description: Apply this project's Rust, Tauri and Svelte code-review conventions when reviewing changes or preparing a PR, and refine them from owner feedback.
---

# Code review

Read available PR inline comments and review bodies when relevant. Apply these rules to changed code, leave unrelated files alone, and respect the user's test constraints. Check `docs/context.md` and the current code when product behavior or conventions are unclear.

## Backend: Rust and Tauri

- Validate input at the Tauri command boundary, including empty or malformed values and applicable length limits. Keep command names, arguments, serialized response types and frontend `invoke` calls consistent.
- Return actionable errors instead of panicking on external input or process failures. Keep user-facing messages in Portuguese and avoid exposing unnecessary process details.
- Keep blocking process and filesystem work off the async runtime where appropriate. Execute external tools with explicit arguments, not a shell; preserve argument separation and validation. Use the bundled yt-dlp resource and its existing configuration safeguards.
- Separate parsing and validation into focused functions where useful. Test meaningful boundaries and failures, including invalid external output. Preserve existing resource paths and packaging behavior when changing tool execution.
- Prefer clear ownership, immutable bindings and early returns; use `Result` and `Option` explicitly where failure or absence is expected. Avoid unnecessary cloning and unchecked `unwrap` on paths that can fail.

## Frontend: Svelte and TypeScript

- Follow the project's Svelte 5 runes and TypeScript conventions. Keep state minimal and related behavior together; represent mutually exclusive states clearly and preserve independent states that can coexist.
- Type Tauri responses and component interfaces. Keep frontend assumptions aligned with Rust serialization and handle rejected commands with useful Portuguese feedback.
- Handle loading, error and empty states; prevent duplicate actions where needed. Guard asynchronous responses against becoming stale when a request is superseded or the component is destroyed, and clean up media and other resources.
- Use semantic controls, labels and accessible status/error feedback. Keep the browser-only experience explicit when a feature requires Tauri. Preserve playback reset and retry behavior when changing search or preview flows.
- Extract components or helpers when they clarify responsibility; avoid splitting files solely to satisfy an arbitrary rule. Comments should explain non-obvious business rules; keep durable implementation context in project docs.

## Delivery and maintenance

- Review the changed code and its boundary contracts, focusing on correctness, regressions and missing meaningful tests. Report findings with file and line references; distinguish verified issues from uncertainties.
- Run checks appropriate to the changes and report results and any limitations. The documented checks are `npm run check`, `npm run build`, and `cargo test --manifest-path src-tauri/Cargo.toml`. For Rust formatting, use `cargo fmt --manifest-path src-tauri/Cargo.toml --check`; format changed Rust code when needed. Do not add a formatter dependency solely for a review; follow existing configuration when available. Report formatting separately from app tests.
- Incorporate reusable owner feedback without source links, preserving scope and rationale. Replace superseded rules and avoid duplicating guidance or turning a specific example into a universal rule.
- Reading feedback alone does not authorize replies, thread resolution, pushing, or merging; follow the user's authorization.
