---
inclusion: fileMatch
fileMatchPattern: "frontend/src/**/*.tsx"
---

# React Idiomatic Patterns for Campfire Frontend

## Core Philosophy: Functional Components and Purity

Modern React is built on functional components that behave like pure functions. Follow the "Rules of Hooks" religiously and embrace composition over inheritance.

## FOUNDATIONAL PATTERNS

### Component Purity Rules
```jsx
// ✅ CORRECT: Pure functional component
function MessageItem({ message, onEdit }) {
  return (
    <div className="message">
      <span>{message.content}</span>
      <button onClick={() => onEdit(message.id)}>Edit</button>
    </div>
  );
}

// ❌ WRONG: Impure component (mutates props)
function MessageItem({ message, onEdit }) {
  message.viewed = true; // Never mutate props!
  return <div>{message.content}</div>;
}
```

### Rules of Hooks (Non-Negotiable)
```jsx
// ✅ CORRECT: Hooks at top level
function ChatRoom({ roomId }) {
  const [messages, setMessages] = useState([]);
  const [isTyping, setIsTyping] = useState(false);
  
  useEffect(() => {
    // Effect logic
  }, [roomId]);
  
  // Conditional logic AFTER hooks
  if (!roomId) return <div>Select a room</div>;
  
  return <div>{/* JSX */}</div>;
}
```

## COMPONENT ARCHITECTURE

### Logic-Presentation Separation
```jsx
// ✅ CORRECT: Custom hook for logic
function useMessages(roomId) {
  const [messages, setMessages] = useState([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState(null);
  
  const sendMessage = useCallback(async (content) => {
    try {
      setIsLoading(true);
      const message = await api.sendMessage(roomId, content);
      setMessages(prev => [...prev, message]);
    } catch (err) {
      setError(err.message);
    } finally {
      setIsLoading(false);
    }
  }, [roomId]);
  
  return { messages, isLoading, error, sendMessage };
}

// Presentational component
function MessageList({ roomId }) {
  const { messages, isLoading, error, sendMessage } = useMessages(roomId);
  
  if (isLoading) return <LoadingSpinner />;
  if (error) return <ErrorMessage error={error} />;
  
  return (
    <div className="message-list">
      {messages.map(message => (
        <MessageItem key={message.id} message={message} />
      ))}
      <MessageComposer onSend={sendMessage} />
    </div>
  );
}
```

## STATE MANAGEMENT MASTERY

### useState vs useReducer Decision Matrix
| Use useState when: | Use useReducer when: |
|-------------------|---------------------|
| Simple state updates | Complex state logic |
| Independent state pieces | Related state updates |
| Boolean/string/number | Objects with multiple fields |

### TanStack Query for Server State
```jsx
// ✅ CORRECT: Server state with TanStack Query
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

function MessageList({ roomId }) {
  const queryClient = useQueryClient();
  
  const { data: messages, isLoading, error } = useQuery({
    queryKey: ['messages', roomId],
    queryFn: () => api.getMessages(roomId),
    staleTime: 30000,
  });
  
  const sendMessageMutation = useMutation({
    mutationFn: api.sendMessage,
    onSuccess: (newMessage) => {
      queryClient.setQueryData(['messages', roomId], (old) => 
        [...(old || []), newMessage]
      );
    },
  });
  
  return <div>{/* JSX */}</div>;
}
```

### Zustand for Simple Global State
```jsx
// ✅ CORRECT: Simple global state
import { create } from 'zustand';

const useChatStore = create((set) => ({
  currentRoom: null,
  notifications: [],
  
  setCurrentRoom: (room) => set({ currentRoom: room }),
  addNotification: (notification) => 
    set((state) => ({ 
      notifications: [...state.notifications, notification] 
    })),
}));
```

## WEBSOCKET PATTERNS

### Real-time Connection Management
```jsx
// ✅ CORRECT: WebSocket hook with reconnection
function useWebSocket(roomId, sessionToken) {
  const [socket, setSocket] = useState(null);
  const [isConnected, setIsConnected] = useState(false);

  useEffect(() => {
    const ws = new WebSocket(
      `ws://localhost:3000/ws?room_id=${roomId}&session_token=${sessionToken}`
    );

    ws.onopen = () => {
      setIsConnected(true);
      setSocket(ws);
    };

    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        // Handle message
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    ws.onclose = () => {
      setIsConnected(false);
      // Simple reconnection after delay
      setTimeout(() => {
        // Reconnect logic
      }, 1000);
    };

    return () => ws.close();
  }, [roomId, sessionToken]);

  return { socket, isConnected };
}
```

### Optimistic UI Updates
```jsx
// ✅ CORRECT: Optimistic updates with rollback
function useOptimisticMessages(roomId) {
  const [optimisticMessages, setOptimisticMessages] = useState(new Map());
  
  const sendMessageOptimistically = useCallback(async (content) => {
    const clientId = crypto.randomUUID();
    const optimisticMessage = {
      id: clientId,
      content,
      status: 'sending',
      created_at: new Date().toISOString(),
    };
    
    // Add optimistic message
    setOptimisticMessages(prev => new Map(prev).set(clientId, optimisticMessage));
    
    try {
      const confirmedMessage = await api.sendMessage(roomId, content, clientId);
      
      // Replace optimistic with confirmed
      setOptimisticMessages(prev => {
        const updated = new Map(prev);
        updated.delete(clientId);
        return updated;
      });
      
      return confirmedMessage;
    } catch (error) {
      // Mark as failed, allow retry
      setOptimisticMessages(prev => {
        const updated = new Map(prev);
        const message = updated.get(clientId);
        if (message) {
          updated.set(clientId, { ...message, status: 'failed', error: error.message });
        }
        return updated;
      });
      
      throw error;
    }
  }, [roomId]);
  
  return { optimisticMessages, sendMessageOptimistically };
}
```

## ERROR HANDLING

### Error Boundaries
```jsx
import { ErrorBoundary } from 'react-error-boundary';

function ErrorFallback({ error, resetErrorBoundary }) {
  return (
    <div className="error-boundary">
      <h2>Something went wrong:</h2>
      <pre>{error.message}</pre>
      <button onClick={resetErrorBoundary}>Try again</button>
    </div>
  );
}

// App-level and component-level error boundaries
function App() {
  return (
    <ErrorBoundary FallbackComponent={ErrorFallback}>
      <ChatInterface />
    </ErrorBoundary>
  );
}
```

## PERFORMANCE PATTERNS

### Strategic Memoization
```jsx
// ✅ CORRECT: Memoize expensive calculations only
function MessageList({ messages, searchTerm }) {
  const filteredMessages = useMemo(() => {
    if (!searchTerm) return messages;
    return messages.filter(message => 
      message.content.toLowerCase().includes(searchTerm.toLowerCase())
    );
  }, [messages, searchTerm]);
  
  return <div>{/* JSX */}</div>;
}

// ❌ WRONG: Unnecessary memoization
function SimpleCounter() {
  const [count, setCount] = useState(0);
  
  const increment = useCallback(() => {
    setCount(c => c + 1);
  }, []); // Unnecessary for simple functions
  
  return <button onClick={increment}>{count}</button>;
}
```

### Virtual Scrolling for Large Lists
```jsx
// ✅ CORRECT: Virtual scrolling for message lists
import { FixedSizeList as List } from 'react-window';

function VirtualizedMessageList({ messages }) {
  const Row = ({ index, style }) => (
    <div style={style}>
      <MessageItem message={messages[index]} />
    </div>
  );

  return (
    <List
      height={600}
      itemCount={messages.length}
      itemSize={80}
      width="100%"
    >
      {Row}
    </List>
  );
}
```

## TESTING PATTERNS

### Component Testing
```jsx
import { render, screen, userEvent } from '@testing-library/react';

test('sends message when form is submitted', async () => {
  const mockSendMessage = jest.fn().mockResolvedValue({ id: '123' });
  const user = userEvent.setup();
  
  render(<MessageComposer onSend={mockSendMessage} />);
  
  const textarea = screen.getByRole('textbox');
  const submitButton = screen.getByRole('button', { name: /send/i });
  
  await user.type(textarea, 'Hello, world!');
  await user.click(submitButton);
  
  expect(mockSendMessage).toHaveBeenCalledWith('Hello, world!');
  expect(textarea).toHaveValue(''); // Should clear after send
});
```

### Custom Hook Testing
```jsx
import { renderHook, act } from '@testing-library/react';

test('useMessages manages state correctly', async () => {
  const { result } = renderHook(() => useMessages('room-123'));
  
  expect(result.current.messages).toEqual([]);
  expect(result.current.loading).toBe(true);
  
  await act(async () => {
    await result.current.sendMessage('Test message');
  });
  
  expect(result.current.messages).toHaveLength(1);
});
```

## ANTI-PATTERNS TO AVOID

### Performance Killers
```jsx
// ❌ WRONG: Objects in render
function MessageList() {
  return (
    <div>
      {messages.map(message => (
        <MessageItem 
          key={message.id}
          style={{ color: 'blue' }} // New object every render!
          message={message}
        />
      ))}
    </div>
  );
}

// ✅ CORRECT: Use CSS classes or stable references
function MessageList() {
  return (
    <div>
      {messages.map(message => (
        <MessageItem 
          key={message.id}
          className="message-item"
          message={message}
        />
      ))}
    </div>
  );
}
```

## KEY TAKEAWAYS

1. **Functional Components**: Use hooks, follow Rules of Hooks religiously
2. **Logic Separation**: Custom hooks for logic, components for presentation
3. **State Management**: Local first, TanStack Query for server state, Zustand for global
4. **Error Handling**: Error boundaries for component errors, try/catch for async
5. **Performance**: Memoize sparingly, avoid objects in render, use virtual scrolling
6. **Testing**: React Testing Library, test behavior not implementation
7. **Real-time**: WebSocket hooks with reconnection, optimistic updates with rollback

When complex React patterns are suggested, respond with: "This adds unnecessary complexity to the React layer. Here's the simple approach that maintains good UX..."