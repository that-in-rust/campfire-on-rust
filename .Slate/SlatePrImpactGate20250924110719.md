Title: PR Impact Gate — Parseltongue-Assisted (SlatePrImpactGate20250924110719)

Purpose
- Quantify change risk per PR via parseltongue (preferred) or grep fallback. Keep repo lean (document-first).

Core commands
- scripts/pt impact
- scripts/pt query uses MessageService
- scripts/pt query calls create_message_with_deduplication
- scripts/pt context MessageService --out
- Fallback: rg -n 'MessageService' src | wc -l

Thresholds
- uses >= 10 or callers >= 10 → broaden tests & deepen review

CI approach (optional)
- Add a warn-only job: scripts/pt ingest; scripts/pt impact; paste counts into PR
