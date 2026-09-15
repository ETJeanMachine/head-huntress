# Agent Instructions

## Rust documentation

- Add rustdoc comments to every public Rust item so descriptions are available in IntelliSense and generated documentation.
- Document public types, fields, functions, methods, enum variants, and type aliases.

## Git commits

- Do not create commits unless the user explicitly requests one.
- Use the exact commit subject/message requested by the user.
- Add the following co-author trailer when creating a requested commit, filling in your personal information:

  `Co-authored-by: MODEL <email>`

- Do not add extra commit-message commentary or body text unless requested.
- Do not add a `Signed-off-by` trailer unless the user explicitly requests one.
- Do not push commits unless the user explicitly requests it.
- Commits should be small in scope; do not push commits containing unrelated features. Break them up between files as necessary.
- When rewriting history, preserve unrelated commits and avoid folding separate work into the rewritten commit.
