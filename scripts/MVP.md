# Parseltongue Toolkit — MVP

Scope
- Single entrypoint: scripts/pt
- Subcommands (MVP): ingest, latest, open, overview, routes, query (uses|calls|grep), context, impact
- Workspace: ./parseltongue_workspace with per-run analysis_TIMESTAMP dirs and a latest symlink

Executable acceptance criteria (WHEN…THEN…SHALL)
1) Ingestion
- WHEN I run scripts/pt ingest
- THEN it SHALL create parseltongue_workspace/analysis_TIMESTAMP with architecture.html and all_entities.txt, and update parseltongue_workspace/latest

2) Query (uses/calls)
- WHEN I run scripts/pt query uses MessageService
- THEN it SHALL print usage sites (or empty output) without error

3) Context export
- WHEN I run scripts/pt context MessageService --out
- THEN it SHALL write context_MessageService.txt into parseltongue_workspace/latest and print the file path

4) Overview and routes
- WHEN I run scripts/pt overview
- THEN it SHALL print key paths including latest viz and entities list
- WHEN I run scripts/pt routes --format table
- THEN it SHALL print method/path/handler rows (best-effort; no crash if patterns are missing)

5) Impact snapshot
- WHEN I run scripts/pt impact
- THEN it SHALL print a table of symbols with uses/callers counts in under 5 seconds on this repo

6) Safety and idempotence
- WHEN I run scripts/pt ingest multiple times without --force
- THEN it SHALL generate new analysis_TIMESTAMP directories and update latest without mutating prior snapshots

Performance budget
- Ingest time: <= 30s (observed << 1s here)
- query/context/impact: <= 5s each on this repo

Usage quickstart
- Ingest and visualize:
  - scripts/pt ingest
  - scripts/pt open --open
- Orientation:
  - scripts/pt overview
  - scripts/pt routes --format table
- Token-efficient lookups:
  - scripts/pt query uses MessageService
  - scripts/pt query calls create_message_with_deduplication
  - scripts/pt query grep MessageService
- Context for prompts:
  - scripts/pt context MessageService --out
- Impact pre-check:
  - scripts/pt impact

Notes
- Parseltongue binary is expected at ./parseltongue or ./parseltongue_workspace/parseltongue
- Outputs are written to ./parseltongue_workspace; previous snapshots preserved and “latest” updated