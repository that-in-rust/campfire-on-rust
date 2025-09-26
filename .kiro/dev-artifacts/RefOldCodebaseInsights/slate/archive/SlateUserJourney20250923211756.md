# Developer User Journeys + JTBD — Parseltongue-Aided Toolkit
Date: 2025-09-23 21:17:56

Purpose
- Capture key developer journeys (JTBD) in this repo and design a parseltongue-aided script toolkit that accelerates each journey while reducing LLM token usage.
- Scripts should follow our principles in .kiro/steering/design101-tdd-architecture-principles.md (TDD, small/cohesive units, observability, safety).

Guiding principles (from design101-tdd-architecture-principles.md)
- Design-first: clarify intent, inputs/outputs, and observable artifacts before implementation.
- TDD: start with small, verifiable steps; iterate with feedback.
- Simplicity and cohesion: one script = one job well; compose larger flows from small commands.
- Observability: each script prints concise, actionable outputs with stable paths for downstream tooling.
- Safety: idempotent where possible; never destroy user data; dry-run options for risky ops.

Developer personas
- New Contributor: needs fast orientation, route overview, component contexts, and examples.
- Feature Builder: needs impact assessment, call graphs, and focused contexts for affected services/handlers.
- Bugfixer: needs quick caller/usage tracing, reproduction context, and narrow patches.
- Refactorer: needs blast-radius estimates, dependency maps, and route/contract checks.
- Reviewer/Tech Lead: needs crisp diffs, impact signals, and navigable architecture views.

Top JTBD journeys and friction points
1) Onboard and understand the architecture
- Pain: jumping across files and guessing entry points
- Success: 10–15 minutes to mental model + key routes + visual map

2) Start a feature (small slice)
- Pain: unknown side effects, finding where to plug in
- Success: scoped change list, callers, and tests to touch

3) Fix a bug quickly
- Pain: “who calls this?” and “who depends on this type?”
- Success: call/usage traces in minutes, minimal diff

4) Refactor safely
- Pain: hidden dependencies, brittle interfaces
- Success: quantified blast radius, thresholds to demand extra tests

5) Review with confidence
- Pain: vague PR descriptions, missing risk markers
- Success: impact numbers and contexts attached to each PR

Toolkit design (scripts align to journeys; all use ./parseltongue_workspace/)
- Common behaviors
  - Ensure ./parseltongue_workspace exists; create if missing
  - Auto-pick latest snapshot; provide flags to force new ingest
  - Print short, token-efficient outputs (counts, file paths, concise summaries)

A) Orientation (Onboarding)
- pt: latest | ingest | open | overview
  - pt ingest: build FILE:-header dump from src/ (and tests if present), ingest, generate:
    - analysis_TIMESTAMP/architecture.html
    - analysis_TIMESTAMP/all_entities.txt
    - context_*.{txt,json} for core components (configurable)
    - latest symlink
  - pt open: print or open latest architecture.html
  - pt overview: echo “Where to look” with paths for routes (src/main.rs), handlers, and services
- LLM token saver: provide 1–2 human-readable context files (MessageService, AuthService) plus the route inventory path instead of raw code.

B) Feature slice start
- pt impact --entities "MessageService,RoomService" --functions "create_message_with_deduplication"
  - Output: uses counts, callers counts, top N callers, and short human context snippets
- pt routes --format table
  - Output: method, path, handler, feature flags (ready to paste into docs/PR)
- pt context <Entity>
  - Output: 30–50 line, human summary; optional --json

C) Bugfix quick trace
- pt calls <function> [--top N]
  - Output: sorted caller locations (file:line minimal), to narrow reproduction
- pt uses <TypeOrTrait> [--top N]
  - Output: usage sites to uncover unexpected dependencies
- pt grep-graph <regex>
  - Output: filter of entity names from graph to discover exact identifiers for subsequent queries

D) Refactor with safety
- pt diff-impact --base <ref> --head <ref>
  - Output: list of changed files → inferred entities → uses/calls counts; risk score with thresholds
- pt pr (PR Impact Gate)
  - Output: aggregated impact summary and generate-context snippets for touched entities

E) Review assist
- pt summarize --entities <...> --functions <...>
  - Output: one-page summary for PR description (impact + routes touched + contexts)

Mapping journeys → scripts
- Onboard: pt ingest → pt open → pt overview → pt routes
- Feature: pt impact → pt context → pt routes
- Bugfix: pt calls / pt uses → pt context
- Refactor: pt diff-impact → pt pr
- Review: pt summarize → attach outputs to PR

Example outputs (token-efficient)
- uses MessageService = 4; calls create_message_with_deduplication = 17
- Top callers (3):
  - src/handlers/messages.rs: create_message (line …)
  - src/handlers/websocket.rs: handle_incoming_message (line …)
  - src/services/… (if any)
- Context paths to share:
  - parseltongue_workspace/analysis_YYYYMMDDHHMMSS/context_MessageService.txt
  - parseltongue_workspace/analysis_YYYYMMDDHHMMSS/context_AuthService.txt

Success metrics
- Reduce time-to-orientation to < 15 minutes for new devs
- For features/bugfixes: produce impact counts and contexts in < 90 seconds
- For refactors: produce risk score and top dependencies in < 2 minutes

Safety and observability (principle alignment)
- Idempotent: pt ingest reuses latest unless --force-new specified
- Clear output: stable paths and minimal, copy-pastable summaries
- Guardrails: pt pr flags high risk (uses > 10 or callers > 10) and suggests tests/reviewers

Next steps (after approval)
- Implement scripts in scripts/ (pt entrypoint plus subcommands)
- Unit-test critical functions (argument parsing, file detection)
- Document usage in Slate-AllInOne-20250923212530.md and README