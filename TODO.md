# TODO — MVP tracker

**MVP definition:** an end-to-end CLI that fetches jobs from job boards (YC and
Greenhouse to start), assesses jobs and stores them, allows resume rewriting, and
gives the user tools to track application processes.

Suggested order: 1 → 2 → 3 → 4; section 5 is independent and can slot in after 3.

## Done

- **Domain logic (`src/logic/`)** — normalized `Job`/`RawJob`, identity + refresh
  rules, candidate profile, evidence-validated `ResumePlan`, and `rules.rs` with
  deterministic hard constraints, weighted semantic scoring, and the
  accept/reject/review decision flow.
- **Ports (`src/ports/`)** — `assessment::SemanticAssessor` (Jev-first),
  `llm::LlmProvider` (generation only), `job_store::JobStore`, `job_source::JobSource`,
  `renderer::ResumeRenderer`.
- **Adapters (`src/adapters/`)**
  - `assessment/typesafe.rs` — TypeSafe Jev assessor: batched Noul questions,
    calibrated probability → Match/NoMatch/Uncertain, retry w/ backoff. Fixture-tested.
  - `llm/openrouter.rs` — assessment (prompt-and-parse fallback) + resume plan
    generation with local validation. Fixture-tested.
  - `job-sources/greenhouse.rs` — public boards API, escaped-HTML handling,
    raw-payload capture for replay. Parser fixture-tested; fetching not live-tested.
  - `storage/` — SQLite (`identity_key` unique dedup, JSON columns) and in-memory
    stores behind `JobStore`. Tested against `sqlite::memory:`.
  - `rendering/html.rs` + `templates/resume.html` — validated plan → HTML, incl.
    real-template integration test.
- **Application services (`src/application/`)** — store-backed `Ingester`
  (dedupe + refresh), `JobEvaluator` (works with any `SemanticAssessor`),
  `ResumeGenerator` (deterministic baseline + LLM plan validation), `ReviewQueue`.
- **Infra** — `.env` keys configured (TypeSafe + OpenRouter + default models),
  config examples (`config/rules.example.yaml`, `config/profile.example.yaml`),
  vendored TypeSafe skill (`.agents/skills/typesafe-ai/SKILL.md`), AGENTS.md
  architecture notes. `cargo test` suite (39 tests) needs no network or credentials.

## 1. CLI end-to-end wiring — **blocking, do first**

- [ ] **Config loading**: parse YAML into domain types — rules
      (`config/rules.example.yaml` shape), profile, and a new sources list
      (Greenhouse tokens + YC config). Tracked examples, real files ignored.
- [ ] **Provider selection**: choose assessor (`typesafe` default | `openrouter`)
      and LLM model from config/env.
- [ ] `sync` command → sources → `RawJob` persistence → normalize → `Ingester` → SQLite.
- [ ] `evaluate` command → load pending jobs → `JobEvaluator` + assessor →
      persist results → route `Review` items to the review queue.
- [ ] `resume` command → job + profile → LLM plan → validate → render
      `templates/resume.html` to an output path.
- [ ] `review` command → list/approve/reject queued jobs.
- [ ] Async runtime setup in `main.rs`/`cli.rs` (thin entry point stays thin;
      workflows belong in `application/`).
- [ ] End-to-end smoke test with mock source + mock assessor (no live network).

## 2. Job sources

- [ ] **YC / Work at a Startup adapter** — research the feed (no official public
      API; likely HTML or undocumented JSON), then implement `JobSource` like Greenhouse.
- [ ] **Raw job persistence** — `raw_jobs` table + `JobStore::insert_raw`; store
      before normalizing (replayable parsing; decided, not yet built).
- [ ] Live-test Greenhouse fetch against a real board (`#[ignore]` test or manual run).
- [ ] Politeness/robustness pass: per-source rate limits, error isolation (one bad
      board shouldn't kill a sync run).

## 3. Assessment + storage

- [ ] **Persist evaluation outcomes** — evaluations table (decision, score, per-rule
      results JSON, timestamp) + store methods or a small dedicated port; currently
      `FilterResult` is in-memory only.
- [ ] Persist review queue state (currently `ReviewQueue` is in-memory).
- [ ] Optional: `Score`-primitive rule style for graded ("how much") preferences.

## 4. Resume rewriting

- [ ] **Profile loading from config** — `CandidateProfile` from
      `config/profile.yaml` (verified achievements with IDs, skills).
- [ ] Extend profile model + template context with contact info, education,
      projects (renderer currently passes null/empty for those).
- [ ] Live-test the OpenRouter plan → validate → render path once; tune
      `DEFAULT_MAX_TOKENS` if plans get truncated.

## 5. Application tracking — **greenfield**

- [ ] Domain model: application status pipeline (e.g., interested → applied →
      interview → offer/rejected) with timestamps and notes, linked to job IDs.
- [ ] Storage: applications table + port methods.
- [ ] CLI: `apply`/`status`/`update` style commands to record and list progress.

## Deferred / non-MVP

- `serve` command (future GUI/WebUI shell — port already split for this).
- Glassdoor adapter (`src/adapters/job-sources/glassdoor.rs` is an empty placeholder).
- Jev evidence-selection question (restore provenance quotes via a second
  Choice question over JD spans).
- Jev-via-OpenRouter as an explicit assessor option (already works by config —
  `OpenRouterLlm` implements `SemanticAssessor` — just needs the selection knob).

## Housekeeping

- [X] Rename `src/application/evalutator.rs` → `evaluator.rs` (misspelled file,
      currently fixed with `#[path]`).
- [ ] Delete the empty `glassdoor.rs` placeholder or leave until section 2 expands.
- [ ] Consider pinning `reqwest` to 0.12 to dedupe with `openrouter-rs`'s copy
      (tree currently compiles two reqwest versions).
