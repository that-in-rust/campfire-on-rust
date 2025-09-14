---
inclusion: fileMatch
fileMatchPattern: "frontend/src/**/*.tsx"
---

# React Real-time Rules

## WEBSOCKET PATTERNS

- **useWebSocket hook** owns reconnect/backoff logic
- **Single WebSocket connection** per user session
- **Room-based subscriptions** - subscribe/unsubscribe to rooms
- **Automatic reconnection** with exponential backoff
- **Connection state management** in custom hook

## MESSAGE LIST PATTERNS

- **Virtualize message lists** for performance with large message counts
- **Preserve scroll position** during updates and new messages
- **Intersection Observer** for scroll anchoring
- **Optimistic UI updates** with rollback on failure
- **5-minute threading windows** for message grouping

## STATE MANAGEMENT

- **TanStack Query** for server state management
- **React state** for UI state only
- **No cross-tab coordination** - each tab manages its own state
- **Optimistic updates** with server reconciliation
- **Simple error boundaries** for graceful failure handling

## REAL-TIME FEATURES

- **Typing indicators** with debounced input
- **Presence tracking** with heartbeat
- **Message broadcasting** with optimistic UI
- **Sound playback** for /play commands
- **Push notifications** via service worker

## PERFORMANCE PATTERNS

- **React.memo** for expensive components
- **useMemo/useCallback** for expensive computations
- **Lazy loading** for non-critical components
- **Debounced inputs** for search and typing
- **Efficient re-renders** through proper dependency arrays

## ACCESSIBILITY

- **Semantic HTML** for screen readers
- **Keyboard navigation** for all interactive elements
- **ARIA labels** for dynamic content
- **Focus management** for modal dialogs
- **Color contrast** compliance

When complex React patterns are suggested, respond with: "This adds unnecessary complexity to the React layer. Here's the simple approach that maintains good UX..."