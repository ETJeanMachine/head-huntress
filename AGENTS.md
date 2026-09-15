# Agent Instructions

## Rust documentation

- Add rustdoc comments to every public Rust item so descriptions are available in IntelliSense and generated documentation.
- Document public types, fields, functions, methods, enum variants, and type aliases.

## Git commits

- Do not create commits unless the user explicitly requests one.
- Use the exact commit subject/message requested by the user. If not requested, keep the commit short and concise and under 50 characters.
- Add the following co-author trailer when creating a requested commit, filling in your personal information:

  `Co-authored-by: MODEL <email>`

- Do not add extra commit-message commentary or body text unless requested. If requested; keep it short (around 72 characters maximum).
- Do not add a `Signed-off-by` trailer unless the user explicitly requests one.
- Do not push commits unless the user explicitly requests it.
- Commits should be small in scope; do not push commits containing unrelated features. Break them up between files as necessary.
- When rewriting history, preserve unrelated commits and avoid folding separate work into the rewritten commit.

## Explanations and code reviews

- Keep explanations concise and proportional to the code being reviewed.
- For small files, summarize the purpose, key design choices, and important caveats without writing an essay unless the user asks for depth.

## Architecture

- Keep `src/main.rs` as a thin startup entry point. CLI behavior belongs in `src/cli.rs`; future GUI/API startup should remain another interface, not domain logic.
- `src/logic/` contains domain types and deterministic business rules. `src/application/` coordinates use cases. `src/ports/` defines application-owned interfaces. `src/adapters/` implements external integrations.
- Do not leak SDK or transport types into `logic` or `application`; adapters translate them into domain types.
- LLM providers implement `ports::llm::LlmProvider`. Keep prompts, structured assessments, resume-plan validation, and evidence provenance in application/domain code.
- Hard job constraints are deterministic gates. Semantic LLM assessments are weighted preferences and must retain confidence, explanation, and evidence; uncertain results go to review.
- Resume generation may reframe verified profile evidence but must not invent claims. Validate LLM-generated plans before rendering `templates/resume.html`.
- Keep secrets in ignored `.env` files. Track only example configuration such as `config/*.example.yaml`; never hardcode provider keys.
- Prefer unit tests with fixtures and mock ports. Do not require live scraper or LLM credentials for the default test suite.
