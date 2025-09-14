---
inclusion: always
---

# Campfire-on-Rust: Anti-Coordination Guardrails

## FORBIDDEN PATTERNS (Reject with rationale)

**These patterns are BANNED and must be rejected immediately:**

- **NO coordination layers, coordinators, or event buses**
- **NO distributed transactions, sagas, or event sourcing** 
- **NO circuit breakers, retry queues, or complex error recovery**
- **NO cross-tab coordination or global state synchronization**
- **NO microservices, service mesh, or distributed architecture**
- **NO message queues, event streams, or async coordination**
- **NO complex state machines or coordination protocols**

## MANDATORY SIMPLICITY PATTERNS

**Use these patterns exclusively:**

- **Direct SQLite operations** - Simple INSERT/UPDATE/SELECT queries
- **Basic WebSocket broadcasting** - Direct room-based message sending
- **Rails-style session management** - Simple cookie-based authentication
- **Simple error handling** - Basic Result<T, E> with user-friendly messages
- **Direct function calls** - No async coordination between components
- **Single binary deployment** - No orchestration or service discovery

## COMPLEXITY LIMITS

- **Maximum 50 total files** in entire codebase (backend + frontend)
- **No file over 500 lines** - Split large files into smaller modules
- **Maximum 3 async operations per request** - Keep request handling simple
- **No more than 2 levels of error handling** - Avoid nested Result chains
- **Single database connection pool** - No distributed data management

## RAILS PARITY RULE

- **If Rails doesn't do it, we don't do it** - Use Rails as the complexity ceiling
- **Replicate Rails patterns exactly** - Don't "improve" on proven Rails behavior
- **Evidence-based additions only** - New patterns require Rails precedent
- **Simple beats clever** - Choose obvious solutions over optimized ones

When any coordination pattern is suggested, respond with: "This violates the anti-coordination mandate. Rails doesn't need this pattern, so neither do we. Here's the simple Rails-equivalent approach instead..."