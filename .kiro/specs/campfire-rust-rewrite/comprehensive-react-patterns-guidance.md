# Comprehensive React Idiomatic Patterns Guidance for Campfire Frontend

## Overview

This document synthesizes comprehensive React idiomatic patterns from extensive reference materials to guide the Campfire frontend development. The patterns are organized by complexity and application domains, ensuring maintainable, performant, and test-driven React applications.

Based on thorough analysis of 424+ lines of React patterns documentation, this guide focuses on modern React development centered around functional components, Hooks, and declarative programming principles.

## Table of Contents

1. [Foundational Philosophy](#foundational-philosophy)
2. [Component Architecture Patterns](#component-architecture-patterns)
3. [State Management Mastery](#state-management-mastery)
4. [Side Effects and Async Operations](#side-effects-and-async-operations)
5. [Error Handling and Resilience](#error-handling-and-resilience)
6. [Project Structure and Organization](#project-structure-and-organization)
7. [Test-Driven Development Integration](#test-driven-development-integration)
8. [Performance Optimization Patterns](#performance-optimization-patterns)
9. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)

---

## Foundational Philosophy

### 1. Functional Components and Purity

**The Bedrock**: Modern React is built on functional components that behave like pure functions.

#### 1.1 Component Purity Rules
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

#### 1.2 The Rules of Hooks (Non-Negotiable)
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

// ❌ WRONG: Conditional hooks
function ChatRoom({ roomId }) {
  if (!roomId) return <div>Select a room</div>;
  
  const [messages, setMessages] = useState([]); // Hook after early return!
  return <div>{/* JSX */}</div>;
}
```

### 2. Composition Over Inheritance

**Pattern**: Build UIs by composing independent, reusable components.

```jsx
// ✅ CORRECT: Composition pattern
function MessageList({ children, header }) {
  return (
    <div className="message-list">
      {header && <div className="header">{header}</div>}
      <div className="messages">{children}</div>
    </div>
  );
}

// Usage
<MessageList header={<RoomHeader room={currentRoom} />}>
  {messages.map(msg => <MessageItem key={msg.id} message={msg} />)}
</MessageList>
```

---

## Component Architecture Patterns

### 1. Single Responsibility Principle

**Rule**: Each component should have only one reason to change.

```jsx
// ❌ WRONG: Monolithic component
function ChatInterface() {
  const [messages, setMessages] = useState([]);
  const [users, setUsers] = useState([]);
  const [currentRoom, setCurrentRoom] = useState(null);
  const [isTyping, setIsTyping] = useState(false);
  
  // 100+ lines of mixed concerns...
  
  return (
    <div>
      {/* Complex JSX mixing all concerns */}
    </div>
  );
}

// ✅ CORRECT: Decomposed components
function ChatInterface() {
  return (
    <div className="chat-interface">
      <RoomSelector />
      <MessageList />
      <MessageComposer />
      <UserList />
    </div>
  );
}
```

### 2. Logic-Presentation Separation

**Pattern**: Use custom hooks for logic, components for presentation.

```jsx
// Custom hook (logic)
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
  
  useEffect(() => {
    // Subscribe to real-time messages
    const unsubscribe = api.subscribeToMessages(roomId, (newMessage) => {
      setMessages(prev => [...prev, newMessage]);
    });
    
    return unsubscribe;
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

### 3. Advanced Composition Patterns

#### 3.1 Compound Components
```jsx
// Compound component pattern for flexible APIs
function Tabs({ children, defaultTab }) {
  const [activeTab, setActiveTab] = useState(defaultTab);
  
  return (
    <TabsContext.Provider value={{ activeTab, setActiveTab }}>
      <div className="tabs">{children}</div>
    </TabsContext.Provider>
  );
}

function TabList({ children }) {
  return <div className="tab-list">{children}</div>;
}

function Tab({ id, children }) {
  const { activeTab, setActiveTab } = useContext(TabsContext);
  return (
    <button 
      className={`tab ${activeTab === id ? 'active' : ''}`}
      onClick={() => setActiveTab(id)}
    >
      {children}
    </button>
  );
}

function TabPanel({ id, children }) {
  const { activeTab } = useContext(TabsContext);
  return activeTab === id ? <div className="tab-panel">{children}</div> : null;
}

// Usage
<Tabs defaultTab="messages">
  <TabList>
    <Tab id="messages">Messages</Tab>
    <Tab id="files">Files</Tab>
  </TabList>
  <TabPanel id="messages">
    <MessageList />
  </TabPanel>
  <TabPanel id="files">
    <FileList />
  </TabPanel>
</Tabs>
```

#### 3.2 Provider Pattern
```jsx
// Theme provider for Campfire
function ThemeProvider({ children }) {
  const [theme, setTheme] = useState('light');
  
  const toggleTheme = useCallback(() => {
    setTheme(prev => prev === 'light' ? 'dark' : 'light');
  }, []);
  
  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      <div className={`app theme-${theme}`}>
        {children}
      </div>
    </ThemeContext.Provider>
  );
}

// Usage in any component
function Header() {
  const { theme, toggleTheme } = useContext(ThemeContext);
  
  return (
    <header>
      <h1>Campfire</h1>
      <button onClick={toggleTheme}>
        Switch to {theme === 'light' ? 'dark' : 'light'} mode
      </button>
    </header>
  );
}
```
-
--

## State Management Mastery

### 1. State Structure Principles

#### 1.1 Minimalism (Single Source of Truth)
```jsx
// ❌ WRONG: Redundant state
function UserProfile() {
  const [firstName, setFirstName] = useState('');
  const [lastName, setLastName] = useState('');
  const [fullName, setFullName] = useState(''); // Redundant!
  
  useEffect(() => {
    setFullName(`${firstName} ${lastName}`); // Extra render cycle
  }, [firstName, lastName]);
  
  return <div>{fullName}</div>;
}

// ✅ CORRECT: Derived state
function UserProfile() {
  const [firstName, setFirstName] = useState('');
  const [lastName, setLastName] = useState('');
  
  // Compute during render
  const fullName = `${firstName} ${lastName}`;
  
  return <div>{fullName}</div>;
}
```

#### 1.2 State Colocation
```jsx
// ✅ CORRECT: Keep state close to where it's used
function ChatRoom() {
  return (
    <div>
      <MessageList /> {/* Messages state lives here */}
      <UserList />    {/* User state lives here */}
    </div>
  );
}

// Only lift state when sharing is needed
function App() {
  const [currentUser, setCurrentUser] = useState(null); // Shared across app
  
  return (
    <UserContext.Provider value={{ currentUser, setCurrentUser }}>
      <ChatRoom />
    </UserContext.Provider>
  );
}
```

### 2. Local State: useState vs useReducer

#### 2.1 useState for Simple State
```jsx
function MessageComposer() {
  const [content, setContent] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  
  const handleSubmit = async () => {
    setIsSubmitting(true);
    try {
      await sendMessage(content);
      setContent('');
    } finally {
      setIsSubmitting(false);
    }
  };
  
  return (
    <form onSubmit={handleSubmit}>
      <textarea 
        value={content} 
        onChange={(e) => setContent(e.target.value)}
        disabled={isSubmitting}
      />
      <button type="submit" disabled={isSubmitting}>
        Send
      </button>
    </form>
  );
}
```

#### 2.2 useReducer for Complex State
```jsx
// Complex message state with multiple transitions
const messageReducer = (state, action) => {
  switch (action.type) {
    case 'SEND_START':
      return { ...state, status: 'sending', error: null };
    case 'SEND_SUCCESS':
      return { 
        ...state, 
        status: 'sent', 
        id: action.payload.id,
        timestamp: action.payload.timestamp 
      };
    case 'SEND_FAILURE':
      return { ...state, status: 'failed', error: action.payload.error };
    case 'RETRY':
      return { ...state, status: 'sending', error: null };
    default:
      return state;
  }
};

function MessageItem({ initialMessage }) {
  const [messageState, dispatch] = useReducer(messageReducer, {
    ...initialMessage,
    status: 'draft',
    error: null
  });
  
  const sendMessage = async () => {
    dispatch({ type: 'SEND_START' });
    try {
      const result = await api.sendMessage(messageState.content);
      dispatch({ type: 'SEND_SUCCESS', payload: result });
    } catch (error) {
      dispatch({ type: 'SEND_FAILURE', payload: { error: error.message } });
    }
  };
  
  return (
    <div className={`message message--${messageState.status}`}>
      <p>{messageState.content}</p>
      {messageState.status === 'failed' && (
        <div>
          <span className="error">{messageState.error}</span>
          <button onClick={() => dispatch({ type: 'RETRY' })}>Retry</button>
        </div>
      )}
    </div>
  );
}
```

### 3. UI State vs Server State

#### 3.1 UI State (Client State)
```jsx
// UI state: owned by client, synchronous
function MessageComposer() {
  const [isExpanded, setIsExpanded] = useState(false);
  const [draft, setDraft] = useState('');
  const [showEmojiPicker, setShowEmojiPicker] = useState(false);
  
  return (
    <div className={`composer ${isExpanded ? 'expanded' : ''}`}>
      {/* UI controls */}
    </div>
  );
}
```

#### 3.2 Server State with TanStack Query
```jsx
// ✅ CORRECT: Use TanStack Query for server state
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

function MessageList({ roomId }) {
  const queryClient = useQueryClient();
  
  // Server state management
  const { 
    data: messages, 
    isLoading, 
    error 
  } = useQuery({
    queryKey: ['messages', roomId],
    queryFn: () => api.getMessages(roomId),
    staleTime: 30000, // 30 seconds
  });
  
  const sendMessageMutation = useMutation({
    mutationFn: api.sendMessage,
    onSuccess: (newMessage) => {
      // Optimistic update
      queryClient.setQueryData(['messages', roomId], (old) => 
        [...(old || []), newMessage]
      );
    },
  });
  
  if (isLoading) return <LoadingSpinner />;
  if (error) return <ErrorMessage error={error} />;
  
  return (
    <div>
      {messages?.map(message => (
        <MessageItem key={message.id} message={message} />
      ))}
      <MessageComposer onSend={sendMessageMutation.mutate} />
    </div>
  );
}

// ❌ WRONG: Manual server state management
function MessageList({ roomId }) {
  const [messages, setMessages] = useState([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState(null);
  
  useEffect(() => {
    // Manual fetch logic - error-prone and complex
    setIsLoading(true);
    api.getMessages(roomId)
      .then(setMessages)
      .catch(setError)
      .finally(() => setIsLoading(false));
  }, [roomId]);
  
  // Missing: caching, background refetch, optimistic updates, etc.
}
```

### 4. Global State Decision Framework

#### 4.1 React Context (Low-frequency updates)
```jsx
// Theme, auth status, user preferences
const AuthContext = createContext();

function AuthProvider({ children }) {
  const [user, setUser] = useState(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  
  return (
    <AuthContext.Provider value={{ user, isAuthenticated, setUser, setIsAuthenticated }}>
      {children}
    </AuthContext.Provider>
  );
}
```

#### 4.2 Zustand (Simple global store)
```jsx
import { create } from 'zustand';

// Simple, performant global state
const useChatStore = create((set, get) => ({
  currentRoom: null,
  notifications: [],
  
  setCurrentRoom: (room) => set({ currentRoom: room }),
  
  addNotification: (notification) => 
    set((state) => ({ 
      notifications: [...state.notifications, notification] 
    })),
    
  removeNotification: (id) =>
    set((state) => ({
      notifications: state.notifications.filter(n => n.id !== id)
    })),
}));

// Usage
function RoomSelector() {
  const { currentRoom, setCurrentRoom } = useChatStore();
  
  return (
    <select 
      value={currentRoom?.id || ''} 
      onChange={(e) => setCurrentRoom(rooms.find(r => r.id === e.target.value))}
    >
      {rooms.map(room => (
        <option key={room.id} value={room.id}>{room.name}</option>
      ))}
    </select>
  );
}
```

#### 4.3 Redux Toolkit (Complex enterprise apps)
```jsx
// For large, complex applications with many developers
import { createSlice, configureStore } from '@reduxjs/toolkit';

const messagesSlice = createSlice({
  name: 'messages',
  initialState: {
    byRoom: {},
    loading: false,
    error: null,
  },
  reducers: {
    fetchMessagesStart: (state) => {
      state.loading = true;
      state.error = null;
    },
    fetchMessagesSuccess: (state, action) => {
      const { roomId, messages } = action.payload;
      state.byRoom[roomId] = messages;
      state.loading = false;
    },
    addMessage: (state, action) => {
      const { roomId, message } = action.payload;
      if (!state.byRoom[roomId]) state.byRoom[roomId] = [];
      state.byRoom[roomId].push(message);
    },
  },
});

const store = configureStore({
  reducer: {
    messages: messagesSlice.reducer,
  },
});
```

---

## Side Effects and Async Operations

### 1. useEffect Patterns

#### 1.1 Dependency Array Management
```jsx
function MessageList({ roomId, userId }) {
  const [messages, setMessages] = useState([]);
  
  // ✅ CORRECT: All dependencies included
  useEffect(() => {
    const fetchMessages = async () => {
      const data = await api.getMessages(roomId, userId);
      setMessages(data);
    };
    
    fetchMessages();
  }, [roomId, userId]); // Both dependencies included
  
  // ✅ CORRECT: Empty dependency array for one-time setup
  useEffect(() => {
    const handleKeyboardShortcuts = (e) => {
      if (e.ctrlKey && e.key === 'k') {
        // Open search
      }
    };
    
    document.addEventListener('keydown', handleKeyboardShortcuts);
    return () => document.removeEventListener('keydown', handleKeyboardShortcuts);
  }, []); // Empty array - runs once
  
  return <div>{/* JSX */}</div>;
}
```

#### 1.2 Cleanup Patterns
```jsx
function RealTimeMessages({ roomId }) {
  const [messages, setMessages] = useState([]);
  
  useEffect(() => {
    // WebSocket connection
    const ws = new WebSocket(`ws://localhost:8080/rooms/${roomId}`);
    
    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      setMessages(prev => [...prev, message]);
    };
    
    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
    
    // Cleanup function - essential for preventing memory leaks
    return () => {
      ws.close();
    };
  }, [roomId]);
  
  return <div>{/* JSX */}</div>;
}
```

### 2. Async Data Fetching Patterns

#### 2.1 Three-State Pattern (Manual)
```jsx
function UserProfile({ userId }) {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  
  useEffect(() => {
    const fetchUser = async () => {
      try {
        setLoading(true);
        setError(null);
        const userData = await api.getUser(userId);
        setUser(userData);
      } catch (err) {
        setError(err.message);
      } finally {
        setLoading(false);
      }
    };
    
    fetchUser();
  }, [userId]);
  
  if (loading) return <LoadingSpinner />;
  if (error) return <ErrorMessage error={error} />;
  if (!user) return <div>User not found</div>;
  
  return <div>{user.name}</div>;
}
```

#### 2.2 Custom Hook Abstraction
```jsx
// Reusable data fetching hook
function useApi(url, dependencies = []) {
  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  
  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        setError(null);
        const response = await fetch(url);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const result = await response.json();
        setData(result);
      } catch (err) {
        setError(err.message);
      } finally {
        setLoading(false);
      }
    };
    
    fetchData();
  }, [url, ...dependencies]);
  
  return { data, loading, error };
}

// Usage
function UserProfile({ userId }) {
  const { data: user, loading, error } = useApi(`/api/users/${userId}`, [userId]);
  
  if (loading) return <LoadingSpinner />;
  if (error) return <ErrorMessage error={error} />;
  
  return <div>{user?.name}</div>;
}
```

---

## Error Handling and Resilience

### 1. Error Boundaries

#### 1.1 Declarative Error Handling
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

// App-level error boundary
function App() {
  return (
    <ErrorBoundary
      FallbackComponent={ErrorFallback}
      onError={(error, errorInfo) => {
        console.error('Error caught by boundary:', error, errorInfo);
        // Send to error reporting service
      }}
    >
      <ChatInterface />
    </ErrorBoundary>
  );
}

// Component-level error boundaries
function ChatRoom() {
  return (
    <div>
      <ErrorBoundary fallback={<div>Failed to load messages</div>}>
        <MessageList />
      </ErrorBoundary>
      
      <ErrorBoundary fallback={<div>Failed to load users</div>}>
        <UserList />
      </ErrorBoundary>
    </div>
  );
}
```

### 2. Imperative Error Handling

#### 2.1 Event Handler Errors
```jsx
function MessageComposer() {
  const [content, setContent] = useState('');
  const [error, setError] = useState(null);
  
  const handleSubmit = async (e) => {
    e.preventDefault();
    
    try {
      setError(null);
      await api.sendMessage(content);
      setContent('');
    } catch (err) {
      // Handle errors in event handlers with try/catch
      setError(err.message);
    }
  };
  
  return (
    <form onSubmit={handleSubmit}>
      {error && <div className="error">{error}</div>}
      <textarea 
        value={content}
        onChange={(e) => setContent(e.target.value)}
      />
      <button type="submit">Send</button>
    </form>
  );
}
```

#### 2.2 Async Operation Errors
```jsx
function FileUpload() {
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState(null);
  
  const handleFileUpload = async (file) => {
    try {
      setUploading(true);
      setError(null);
      
      const formData = new FormData();
      formData.append('file', file);
      
      const response = await fetch('/api/upload', {
        method: 'POST',
        body: formData,
      });
      
      if (!response.ok) {
        throw new Error(`Upload failed: ${response.statusText}`);
      }
      
      const result = await response.json();
      console.log('Upload successful:', result);
      
    } catch (err) {
      setError(err.message);
    } finally {
      setUploading(false);
    }
  };
  
  return (
    <div>
      {error && <div className="error">Upload failed: {error}</div>}
      <input 
        type="file" 
        onChange={(e) => handleFileUpload(e.target.files[0])}
        disabled={uploading}
      />
      {uploading && <div>Uploading...</div>}
    </div>
  );
}
```---


## Project Structure and Organization

### 1. Feature-Based Colocation

#### 1.1 Recommended Structure
```
/src
  /components          # Shared, generic UI components
    /Button
      - Button.tsx
      - Button.test.tsx
      - Button.module.css
    /Modal
      - Modal.tsx
      - Modal.test.tsx
  
  /features           # Feature-specific code
    /chat
      /components
        - MessageList.tsx
        - MessageItem.tsx
        - MessageComposer.tsx
      /hooks
        - useMessages.ts
        - useRealTimeUpdates.ts
      /api
        - messagesApi.ts
      /types
        - message.types.ts
      - chat.test.tsx
    
    /rooms
      /components
        - RoomList.tsx
        - RoomSelector.tsx
      /hooks
        - useRooms.ts
      /api
        - roomsApi.ts
    
  /hooks              # Shared, generic hooks
    - useLocalStorage.ts
    - useWebSocket.ts
    - useDebounce.ts
  
  /lib                # Non-React utilities
    - api.ts
    - utils.ts
    - constants.ts
  
  /types              # Shared TypeScript types
    - user.types.ts
    - api.types.ts
```

#### 1.2 Benefits of Feature-Based Structure
- **Reduced Cognitive Load**: All related files in one place
- **Easier Refactoring**: Self-contained features can be modified independently
- **Clear Boundaries**: Prevents accidental coupling between features
- **Simplified Testing**: Feature tests can be comprehensive and isolated

### 2. Naming Conventions

#### 2.1 File Naming Standards
```jsx
// ✅ CORRECT: Consistent naming
MessageList.tsx        // PascalCase for components
useMessages.ts         // camelCase with 'use' prefix for hooks
messagesApi.ts         // camelCase for utilities
message.types.ts       // camelCase with .types suffix
MessageList.test.tsx   // .test suffix for tests
MessageList.module.css // .module suffix for CSS modules

// ❌ WRONG: Inconsistent naming
messageList.tsx        // Should be PascalCase
Messages.hook.ts       // Should be useMessages.ts
api-messages.ts        // Should be messagesApi.ts
```

#### 2.2 Component Organization
```jsx
// ✅ CORRECT: Component with multiple files
/MessageCard
  - MessageCard.tsx      # Main component
  - MessageCard.test.tsx # Tests
  - MessageCard.module.css # Styles
  - index.ts             # Re-export (optional)

// index.ts content
export { MessageCard } from './MessageCard';
export type { MessageCardProps } from './MessageCard';
```

---

## Test-Driven Development Integration

### 1. TDD Workflow: Red-Green-Refactor

#### 1.1 Example: Building a Search Component
```jsx
// Step 1 (RED): Write failing test
import { render, screen, userEvent } from '@testing-library/react';
import { SearchComponent } from './SearchComponent';

test('renders search prompt initially', () => {
  render(<SearchComponent />);
  expect(screen.getByText('Search for messages')).toBeInTheDocument();
});

// Step 2 (GREEN): Minimal implementation
function SearchComponent() {
  return <h2>Search for messages</h2>;
}

// Step 3 (RED): Add interaction test
test('shows loading indicator when searching', async () => {
  const user = userEvent.setup();
  render(<SearchComponent />);
  
  const input = screen.getByRole('textbox');
  const button = screen.getByRole('button', { name: /search/i });
  
  await user.type(input, 'hello');
  await user.click(button);
  
  expect(screen.getByRole('status')).toBeInTheDocument();
});

// Step 4 (GREEN): Add functionality
function SearchComponent() {
  const [query, setQuery] = useState('');
  const [loading, setLoading] = useState(false);
  
  const handleSearch = () => {
    setLoading(true);
    // TODO: Implement search
  };
  
  return (
    <div>
      <h2>Search for messages</h2>
      <input 
        type="text"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <button onClick={handleSearch}>Search</button>
      {loading && <div role="status">Searching...</div>}
    </div>
  );
}

// Step 5 (REFACTOR): Extract custom hook
function useSearch() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  
  const search = useCallback(async () => {
    if (!query.trim()) return;
    
    try {
      setLoading(true);
      setError(null);
      const data = await api.searchMessages(query);
      setResults(data);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }, [query]);
  
  return { query, setQuery, results, loading, error, search };
}

function SearchComponent() {
  const { query, setQuery, results, loading, error, search } = useSearch();
  
  return (
    <div>
      <h2>Search for messages</h2>
      <input 
        type="text"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <button onClick={search}>Search</button>
      {loading && <div role="status">Searching...</div>}
      {error && <div className="error">{error}</div>}
      {results.map(result => (
        <SearchResult key={result.id} result={result} />
      ))}
    </div>
  );
}
```

### 2. Testing Strategies

#### 2.1 Component Testing with RTL
```jsx
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MessageComposer } from './MessageComposer';

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

#### 2.2 Custom Hook Testing
```jsx
import { renderHook, act } from '@testing-library/react';
import { useMessages } from './useMessages';

test('useMessages manages message state correctly', async () => {
  const { result } = renderHook(() => useMessages('room-123'));
  
  expect(result.current.messages).toEqual([]);
  expect(result.current.loading).toBe(true);
  
  // Wait for initial load
  await waitFor(() => {
    expect(result.current.loading).toBe(false);
  });
  
  // Test sending a message
  await act(async () => {
    await result.current.sendMessage('Test message');
  });
  
  expect(result.current.messages).toHaveLength(1);
  expect(result.current.messages[0].content).toBe('Test message');
});
```

#### 2.3 Context Testing
```jsx
import { render, screen } from '@testing-library/react';
import { AuthContext } from './AuthContext';
import { UserProfile } from './UserProfile';

test('shows user name when authenticated', () => {
  const mockUser = { id: '1', name: 'John Doe' };
  
  render(
    <AuthContext.Provider value={{ user: mockUser, isAuthenticated: true }}>
      <UserProfile />
    </AuthContext.Provider>
  );
  
  expect(screen.getByText('John Doe')).toBeInTheDocument();
});

test('shows login prompt when not authenticated', () => {
  render(
    <AuthContext.Provider value={{ user: null, isAuthenticated: false }}>
      <UserProfile />
    </AuthContext.Provider>
  );
  
  expect(screen.getByText('Please log in')).toBeInTheDocument();
});
```

---

## Performance Optimization Patterns

### 1. Memoization (Use Sparingly)

#### 1.1 useMemo for Expensive Calculations
```jsx
function MessageList({ messages, searchTerm }) {
  // ✅ CORRECT: Memoize expensive filtering operation
  const filteredMessages = useMemo(() => {
    if (!searchTerm) return messages;
    
    return messages.filter(message => 
      message.content.toLowerCase().includes(searchTerm.toLowerCase()) ||
      message.author.name.toLowerCase().includes(searchTerm.toLowerCase())
    );
  }, [messages, searchTerm]);
  
  return (
    <div>
      {filteredMessages.map(message => (
        <MessageItem key={message.id} message={message} />
      ))}
    </div>
  );
}
```

#### 1.2 useCallback for Stable References
```jsx
const MemoizedMessageItem = React.memo(MessageItem);

function MessageList({ messages }) {
  // ✅ CORRECT: Stable callback reference for memoized child
  const handleEdit = useCallback((messageId, newContent) => {
    // Edit logic
    updateMessage(messageId, newContent);
  }, []); // No dependencies if updateMessage is stable
  
  return (
    <div>
      {messages.map(message => (
        <MemoizedMessageItem 
          key={message.id} 
          message={message}
          onEdit={handleEdit} // Stable reference prevents re-renders
        />
      ))}
    </div>
  );
}

// ❌ WRONG: Overusing memoization
function SimpleCounter() {
  const [count, setCount] = useState(0);
  
  // Unnecessary memoization for simple calculation
  const doubledCount = useMemo(() => count * 2, [count]);
  
  // Unnecessary callback memoization
  const increment = useCallback(() => setCount(c => c + 1), []);
  
  return (
    <div>
      <p>Count: {count}</p>
      <p>Doubled: {doubledCount}</p>
      <button onClick={increment}>+</button>
    </div>
  );
}
```

### 2. Component Optimization

#### 2.1 React.memo for Pure Components
```jsx
// ✅ CORRECT: Memoize expensive pure components
const MessageItem = React.memo(function MessageItem({ message, onEdit }) {
  return (
    <div className="message">
      <div className="author">{message.author.name}</div>
      <div className="content">{message.content}</div>
      <div className="timestamp">{formatTime(message.createdAt)}</div>
      <button onClick={() => onEdit(message.id)}>Edit</button>
    </div>
  );
});

// Custom comparison for complex props
const MessageItem = React.memo(function MessageItem({ message, onEdit }) {
  // Component implementation
}, (prevProps, nextProps) => {
  // Custom comparison logic
  return prevProps.message.id === nextProps.message.id &&
         prevProps.message.content === nextProps.message.content &&
         prevProps.message.updatedAt === nextProps.message.updatedAt;
});
```

---

## Anti-Patterns to Avoid

### 1. State Management Anti-Patterns

#### 1.1 Direct State Mutation
```jsx
// ❌ WRONG: Mutating state directly
function MessageList() {
  const [messages, setMessages] = useState([]);
  
  const addMessage = (newMessage) => {
    messages.push(newMessage); // Direct mutation!
    setMessages(messages); // React won't detect the change
  };
  
  return <div>{/* JSX */}</div>;
}

// ✅ CORRECT: Immutable updates
function MessageList() {
  const [messages, setMessages] = useState([]);
  
  const addMessage = (newMessage) => {
    setMessages(prev => [...prev, newMessage]); // Create new array
  };
  
  const updateMessage = (id, updates) => {
    setMessages(prev => prev.map(msg => 
      msg.id === id ? { ...msg, ...updates } : msg // Create new object
    ));
  };
  
  return <div>{/* JSX */}</div>;
}
```

#### 1.2 Prop Drilling
```jsx
// ❌ WRONG: Prop drilling through multiple levels
function App() {
  const [user, setUser] = useState(null);
  
  return <ChatInterface user={user} setUser={setUser} />;
}

function ChatInterface({ user, setUser }) {
  return <MessageList user={user} setUser={setUser} />; // Just passing through
}

function MessageList({ user, setUser }) {
  return <MessageItem user={user} setUser={setUser} />; // Still passing through
}

function MessageItem({ user, setUser }) {
  return <div>Message by {user?.name}</div>; // Finally used here
}

// ✅ CORRECT: Use Context API
const UserContext = createContext();

function App() {
  const [user, setUser] = useState(null);
  
  return (
    <UserContext.Provider value={{ user, setUser }}>
      <ChatInterface />
    </UserContext.Provider>
  );
}

function MessageItem() {
  const { user } = useContext(UserContext); // Direct access
  return <div>Message by {user?.name}</div>;
}
```

### 2. useEffect Anti-Patterns

#### 2.1 Missing Dependencies
```jsx
// ❌ WRONG: Missing dependencies
function MessageList({ roomId, userId }) {
  const [messages, setMessages] = useState([]);
  
  useEffect(() => {
    fetchMessages(roomId, userId).then(setMessages);
  }, [roomId]); // Missing userId dependency!
  
  return <div>{/* JSX */}</div>;
}

// ✅ CORRECT: All dependencies included
function MessageList({ roomId, userId }) {
  const [messages, setMessages] = useState([]);
  
  useEffect(() => {
    fetchMessages(roomId, userId).then(setMessages);
  }, [roomId, userId]); // All dependencies included
  
  return <div>{/* JSX */}</div>;
}
```

#### 2.2 Using useEffect for Derived State
```jsx
// ❌ WRONG: useEffect for derived state
function UserProfile({ firstName, lastName }) {
  const [fullName, setFullName] = useState('');
  
  useEffect(() => {
    setFullName(`${firstName} ${lastName}`); // Causes extra render
  }, [firstName, lastName]);
  
  return <div>{fullName}</div>;
}

// ✅ CORRECT: Compute during render
function UserProfile({ firstName, lastName }) {
  const fullName = `${firstName} ${lastName}`; // Computed during render
  
  return <div>{fullName}</div>;
}
```

### 3. Performance Anti-Patterns

#### 3.1 Premature Optimization
```jsx
// ❌ WRONG: Unnecessary memoization everywhere
function SimpleButton({ label, onClick }) {
  const memoizedLabel = useMemo(() => label, [label]); // Unnecessary!
  const memoizedClick = useCallback(onClick, [onClick]); // Unnecessary!
  
  return <button onClick={memoizedClick}>{memoizedLabel}</button>;
}

// ✅ CORRECT: Simple component without memoization
function SimpleButton({ label, onClick }) {
  return <button onClick={onClick}>{label}</button>;
}
```

#### 3.2 Inline Object Creation
```jsx
// ❌ WRONG: Creating objects in render
function MessageList({ messages }) {
  return (
    <div>
      {messages.map(message => (
        <MessageItem 
          key={message.id}
          message={message}
          style={{ color: 'blue', fontSize: '14px' }} // New object every render!
          config={{ showTimestamp: true, allowEdit: false }} // New object every render!
        />
      ))}
    </div>
  );
}

// ✅ CORRECT: Stable references
const MESSAGE_STYLE = { color: 'blue', fontSize: '14px' };
const MESSAGE_CONFIG = { showTimestamp: true, allowEdit: false };

function MessageList({ messages }) {
  return (
    <div>
      {messages.map(message => (
        <MessageItem 
          key={message.id}
          message={message}
          style={MESSAGE_STYLE} // Stable reference
          config={MESSAGE_CONFIG} // Stable reference
        />
      ))}
    </div>
  );
}
```

---

## Campfire-Specific React Patterns

### 1. Real-Time Message Updates

#### 1.1 WebSocket Integration
```jsx
function useRealTimeMessages(roomId) {
  const [messages, setMessages] = useState([]);
  const [connectionStatus, setConnectionStatus] = useState('connecting');
  
  useEffect(() => {
    const ws = new WebSocket(`ws://localhost:8080/rooms/${roomId}`);
    
    ws.onopen = () => setConnectionStatus('connected');
    ws.onclose = () => setConnectionStatus('disconnected');
    ws.onerror = () => setConnectionStatus('error');
    
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      
      switch (data.type) {
        case 'new_message':
          setMessages(prev => [...prev, data.message]);
          break;
        case 'message_updated':
          setMessages(prev => prev.map(msg => 
            msg.id === data.message.id ? data.message : msg
          ));
          break;
        case 'message_deleted':
          setMessages(prev => prev.filter(msg => msg.id !== data.messageId));
          break;
      }
    };
    
    return () => ws.close();
  }, [roomId]);
  
  return { messages, connectionStatus };
}
```

### 2. Typing Indicators

#### 2.1 Debounced Typing Status
```jsx
function useTypingIndicator(roomId, userId) {
  const [isTyping, setIsTyping] = useState(false);
  const [typingUsers, setTypingUsers] = useState(new Set());
  
  const debouncedStopTyping = useMemo(
    () => debounce(() => {
      setIsTyping(false);
      api.sendTypingStatus(roomId, userId, false);
    }, 2000),
    [roomId, userId]
  );
  
  const startTyping = useCallback(() => {
    if (!isTyping) {
      setIsTyping(true);
      api.sendTypingStatus(roomId, userId, true);
    }
    debouncedStopTyping();
  }, [isTyping, roomId, userId, debouncedStopTyping]);
  
  useEffect(() => {
    const handleTypingUpdate = (data) => {
      setTypingUsers(prev => {
        const newSet = new Set(prev);
        if (data.isTyping) {
          newSet.add(data.userId);
        } else {
          newSet.delete(data.userId);
        }
        return newSet;
      });
    };
    
    api.subscribeToTyping(roomId, handleTypingUpdate);
    
    return () => api.unsubscribeFromTyping(roomId, handleTypingUpdate);
  }, [roomId]);
  
  return { isTyping, startTyping, typingUsers };
}
```

### 3. File Upload with Progress

#### 3.1 Upload Progress Tracking
```jsx
function useFileUpload() {
  const [uploads, setUploads] = useState(new Map());
  
  const uploadFile = useCallback(async (file) => {
    const uploadId = crypto.randomUUID();
    
    setUploads(prev => new Map(prev).set(uploadId, {
      file,
      progress: 0,
      status: 'uploading',
      error: null,
    }));
    
    try {
      const formData = new FormData();
      formData.append('file', file);
      
      const response = await fetch('/api/upload', {
        method: 'POST',
        body: formData,
        onUploadProgress: (progressEvent) => {
          const progress = Math.round(
            (progressEvent.loaded * 100) / progressEvent.total
          );
          
          setUploads(prev => {
            const newMap = new Map(prev);
            const upload = newMap.get(uploadId);
            if (upload) {
              newMap.set(uploadId, { ...upload, progress });
            }
            return newMap;
          });
        },
      });
      
      const result = await response.json();
      
      setUploads(prev => {
        const newMap = new Map(prev);
        newMap.set(uploadId, {
          ...newMap.get(uploadId),
          status: 'completed',
          result,
        });
        return newMap;
      });
      
      return result;
      
    } catch (error) {
      setUploads(prev => {
        const newMap = new Map(prev);
        newMap.set(uploadId, {
          ...newMap.get(uploadId),
          status: 'failed',
          error: error.message,
        });
        return newMap;
      });
      
      throw error;
    }
  }, []);
  
  return { uploads, uploadFile };
}
```

---

## Summary

This comprehensive guide provides the essential React patterns needed for the Campfire frontend. The key principles are:

1. **Embrace Functional Programming**: Use pure components and custom hooks
2. **Follow the Rules of Hooks**: Never call hooks conditionally
3. **Separate Concerns**: Logic in hooks, presentation in components
4. **Manage State Correctly**: Distinguish UI state from server state
5. **Handle Errors Gracefully**: Use Error Boundaries and try/catch appropriately
6. **Structure for Scale**: Feature-based colocation and clear naming conventions
7. **Test-Driven Development**: Write tests first, refactor with confidence
8. **Optimize Judiciously**: Profile first, then apply memoization where needed
9. **Avoid Common Pitfalls**: No state mutation, prop drilling, or premature optimization

By following these patterns, the Campfire React frontend will be maintainable, performant, and robust, providing an excellent foundation for the chat application's user interface.---

#
# Advanced Patterns from Complete Analysis

### Test-Driven Development Integration

The complete analysis reveals that TDD is not just about testing—it's a design methodology that naturally guides developers toward idiomatic React patterns.

#### Red-Green-Refactor Cycle for React

```jsx
// 1. RED: Write failing test first
describe('SearchComponent', () => {
  it('should show loading state when searching', async () => {
    render(<SearchComponent />);
    
    const input = screen.getByRole('textbox');
    const button = screen.getByRole('button', { name: /search/i });
    
    await userEvent.type(input, 'test query');
    await userEvent.click(button);
    
    expect(screen.getByRole('status')).toBeInTheDocument();
  });
});

// 2. GREEN: Minimal implementation to pass
const SearchComponent = () => {
  const [query, setQuery] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  
  const handleSearch = () => {
    setIsLoading(true);
  };
  
  return (
    <div>
      <input 
        type="text" 
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <button onClick={handleSearch}>Search</button>
      {isLoading && <div role="status">Loading...</div>}
    </div>
  );
};

// 3. REFACTOR: Extract custom hook
const useSearch = (searchFn) => {
  const [query, setQuery] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [results, setResults] = useState([]);
  const [error, setError] = useState(null);
  
  const search = useCallback(async () => {
    if (!query.trim()) return;
    
    setIsLoading(true);
    setError(null);
    
    try {
      const data = await searchFn(query);
      setResults(data);
    } catch (err) {
      setError(err.message);
    } finally {
      setIsLoading(false);
    }
  }, [query, searchFn]);
  
  return {
    query,
    setQuery,
    results,
    isLoading,
    error,
    search
  };
};

// Refactored component using custom hook
const SearchComponent = ({ onSearch }) => {
  const { query, setQuery, results, isLoading, error, search } = useSearch(onSearch);
  
  return (
    <div>
      <input 
        type="text" 
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <button onClick={search}>Search</button>
      {isLoading && <div role="status">Loading...</div>}
      {error && <div role="alert">{error}</div>}
      {results.map(result => (
        <div key={result.id}>{result.title}</div>
      ))}
    </div>
  );
};
```

### Advanced Composition Patterns

#### Compound Components for Complex UI

```jsx
// Campfire Room Component with Compound Pattern
const Room = ({ children, roomId }) => {
  const [roomState, setRoomState] = useState({
    messages: [],
    users: [],
    typingUsers: [],
    isConnected: false
  });

  const contextValue = useMemo(() => ({
    roomId,
    roomState,
    setRoomState,
    // Derived state
    messageCount: roomState.messages.length,
    userCount: roomState.users.length,
    hasTypingUsers: roomState.typingUsers.length > 0
  }), [roomId, roomState]);

  return (
    <RoomContext.Provider value={contextValue}>
      <div className="room" data-room-id={roomId}>
        {children}
      </div>
    </RoomContext.Provider>
  );
};

// Sub-components with specific responsibilities
Room.Header = ({ children }) => {
  const { roomState } = useContext(RoomContext);
  
  return (
    <header className="room-header">
      <h2>{roomState.name}</h2>
      <UserCount count={roomState.users.length} />
      <ConnectionStatus isConnected={roomState.isConnected} />
      {children}
    </header>
  );
};

Room.Messages = () => {
  const { roomState } = useContext(RoomContext);
  
  return (
    <div className="messages-container">
      <VirtualizedMessageList messages={roomState.messages} />
      <TypingIndicator users={roomState.typingUsers} />
    </div>
  );
};

Room.Input = () => {
  const { roomId } = useContext(RoomContext);
  const { sendMessage } = useWebSocket(roomId);
  
  return (
    <MessageInput 
      onSend={(content) => sendMessage({ 
        type: 'new_message', 
        content,
        timestamp: Date.now()
      })}
      placeholder="Type a message..."
    />
  );
};

// Usage - flexible composition
const ChatRoom = ({ roomId }) => (
  <Room roomId={roomId}>
    <Room.Header>
      <RoomSettings />
    </Room.Header>
    <Room.Messages />
    <Room.Input />
  </Room>
);
```

#### Render Props for Maximum Flexibility

```jsx
// Flexible Message List with Render Props
const MessageList = ({ 
  messages, 
  renderMessage, 
  renderEmpty, 
  renderLoading,
  renderError,
  isLoading = false,
  error = null
}) => {
  if (error) {
    return renderError ? renderError(error) : (
      <div className="error-state" role="alert">
        <p>Failed to load messages: {error.message}</p>
        <button onClick={() => window.location.reload()}>
          Retry
        </button>
      </div>
    );
  }

  if (isLoading) {
    return renderLoading ? renderLoading() : (
      <div className="loading-state">
        <div className="spinner" role="status" aria-label="Loading messages" />
        <p>Loading messages...</p>
      </div>
    );
  }

  if (messages.length === 0) {
    return renderEmpty ? renderEmpty() : (
      <div className="empty-state">
        <p>No messages yet. Start the conversation!</p>
      </div>
    );
  }

  return (
    <div className="message-list" role="log" aria-live="polite">
      {messages.map((message, index) => (
        <div key={message.id} className="message-wrapper">
          {renderMessage ? 
            renderMessage(message, index, messages) : 
            <DefaultMessage message={message} />
          }
        </div>
      ))}
    </div>
  );
};

// Usage with custom rendering logic
const CustomChatRoom = () => {
  const { messages, isLoading, error } = useMessages(roomId);
  
  return (
    <MessageList
      messages={messages}
      isLoading={isLoading}
      error={error}
      renderMessage={(message, index, allMessages) => {
        const showAvatar = index === 0 || 
          allMessages[index - 1].creatorId !== message.creatorId;
        
        const showTimestamp = index % 10 === 0; // Every 10th message
        
        return (
          <Message 
            key={message.id}
            message={message}
            showAvatar={showAvatar}
            showTimestamp={showTimestamp}
            variant={message.creatorId === currentUserId ? 'own' : 'other'}
          />
        );
      }}
      renderEmpty={() => (
        <WelcomeMessage 
          roomName={roomName} 
          onInviteUsers={() => setShowInviteModal(true)}
        />
      )}
      renderError={(error) => (
        <ErrorBoundaryFallback 
          error={error}
          resetError={() => refetch()}
        />
      )}
    />
  );
};
```

### Advanced State Management Patterns

#### Optimistic Updates with Rollback

```jsx
// Advanced optimistic updates with automatic rollback
const useOptimisticMessages = () => {
  const { state, actions } = useApp();
  const { sendMessage } = useWebSocket();
  const [pendingMessages, setPendingMessages] = useState(new Map());
  const rollbackTimeoutRef = useRef(new Map());

  const sendOptimisticMessage = useCallback(async (content) => {
    const tempId = `temp-${Date.now()}-${Math.random()}`;
    const optimisticMessage = {
      id: tempId,
      content,
      creatorId: state.user.id,
      createdAt: new Date().toISOString(),
      status: 'sending',
      isOptimistic: true
    };

    // Add optimistic message immediately
    actions.addMessage(optimisticMessage);
    setPendingMessages(prev => new Map(prev).set(tempId, optimisticMessage));

    // Set rollback timeout (30 seconds)
    const timeoutId = setTimeout(() => {
      actions.updateMessage({
        id: tempId,
        status: 'failed',
        error: 'Message send timeout'
      });
    }, 30000);
    
    rollbackTimeoutRef.current.set(tempId, timeoutId);

    try {
      // Send to server
      const serverMessage = await sendMessage({
        type: 'new_message',
        content,
        clientMessageId: tempId
      });

      // Clear timeout
      const timeout = rollbackTimeoutRef.current.get(tempId);
      if (timeout) {
        clearTimeout(timeout);
        rollbackTimeoutRef.current.delete(tempId);
      }

      // Replace optimistic message with server response
      actions.replaceMessage(tempId, {
        ...serverMessage,
        status: 'sent'
      });
      
      setPendingMessages(prev => {
        const newMap = new Map(prev);
        newMap.delete(tempId);
        return newMap;
      });

    } catch (error) {
      // Clear timeout
      const timeout = rollbackTimeoutRef.current.get(tempId);
      if (timeout) {
        clearTimeout(timeout);
        rollbackTimeoutRef.current.delete(tempId);
      }

      // Mark message as failed with retry option
      actions.updateMessage({
        id: tempId,
        status: 'failed',
        error: error.message,
        canRetry: true
      });
    }
  }, [state.user, actions, sendMessage]);

  const retryMessage = useCallback(async (messageId) => {
    const pendingMessage = pendingMessages.get(messageId);
    if (pendingMessage) {
      // Remove failed message
      actions.deleteMessage(messageId);
      setPendingMessages(prev => {
        const newMap = new Map(prev);
        newMap.delete(messageId);
        return newMap;
      });
      
      // Resend with new optimistic message
      await sendOptimisticMessage(pendingMessage.content);
    }
  }, [pendingMessages, sendOptimisticMessage, actions]);

  // Cleanup timeouts on unmount
  useEffect(() => {
    return () => {
      rollbackTimeoutRef.current.forEach(timeout => clearTimeout(timeout));
    };
  }, []);

  return {
    sendOptimisticMessage,
    retryMessage,
    pendingMessages: Array.from(pendingMessages.values())
  };
};
```

### Advanced Performance Patterns

#### Virtualization with Dynamic Heights

```jsx
// Advanced virtualization for messages with dynamic heights
import { VariableSizeList as List } from 'react-window';

const VirtualizedMessageList = ({ 
  messages, 
  height = 400,
  onLoadMore,
  estimatedItemSize = 80
}) => {
  const listRef = useRef();
  const itemHeights = useRef(new Map());
  const [isAutoScrolling, setIsAutoScrolling] = useState(true);
  const resizeObserver = useRef();

  // Measure item heights dynamically
  const getItemSize = useCallback((index) => {
    return itemHeights.current.get(index) || estimatedItemSize;
  }, [estimatedItemSize]);

  // Set up resize observer for dynamic height measurement
  useEffect(() => {
    resizeObserver.current = new ResizeObserver((entries) => {
      let hasChanged = false;
      
      entries.forEach((entry) => {
        const index = parseInt(entry.target.dataset.index);
        const newHeight = entry.contentRect.height;
        
        if (itemHeights.current.get(index) !== newHeight) {
          itemHeights.current.set(index, newHeight);
          hasChanged = true;
        }
      });
      
      if (hasChanged && listRef.current) {
        listRef.current.resetAfterIndex(0);
      }
    });

    return () => {
      if (resizeObserver.current) {
        resizeObserver.current.disconnect();
      }
    };
  }, []);

  // Message renderer with height measurement
  const MessageRow = useCallback(({ index, style, data }) => {
    const message = data.messages[index];
    const rowRef = useRef();
    
    useEffect(() => {
      if (rowRef.current && resizeObserver.current) {
        rowRef.current.dataset.index = index;
        resizeObserver.current.observe(rowRef.current);
        
        return () => {
          if (resizeObserver.current && rowRef.current) {
            resizeObserver.current.unobserve(rowRef.current);
          }
        };
      }
    }, [index]);
    
    return (
      <div style={style}>
        <div ref={rowRef} data-index={index}>
          <Message 
            message={message}
            currentUserId={data.currentUserId}
            showAvatar={index === 0 || 
              data.messages[index - 1]?.creatorId !== message.creatorId
            }
          />
        </div>
      </div>
    );
  }, []);

  // Memoize message data
  const itemData = useMemo(() => ({
    messages,
    currentUserId: user?.id
  }), [messages, user?.id]);

  // Auto-scroll to bottom for new messages
  useEffect(() => {
    if (isAutoScrolling && messages.length > 0) {
      listRef.current?.scrollToItem(messages.length - 1, 'end');
    }
  }, [messages.length, isAutoScrolling]);

  // Handle scroll events with intersection observer for load more
  const handleScroll = useCallback(({ scrollOffset, scrollUpdateWasRequested }) => {
    if (!scrollUpdateWasRequested) {
      // Check if near bottom for auto-scroll
      const totalHeight = messages.reduce((sum, _, index) => 
        sum + getItemSize(index), 0
      );
      const isNearBottom = scrollOffset > totalHeight - height - 100;
      setIsAutoScrolling(isNearBottom);
      
      // Load more messages when scrolling to top
      if (scrollOffset < 200 && onLoadMore) {
        onLoadMore();
      }
    }
  }, [messages, getItemSize, height, onLoadMore]);

  return (
    <div className="virtualized-message-list">
      <List
        ref={listRef}
        height={height}
        itemCount={messages.length}
        itemSize={getItemSize}
        itemData={itemData}
        onScroll={handleScroll}
        overscanCount={5} // Render extra items for smooth scrolling
      >
        {MessageRow}
      </List>
      
      {!isAutoScrolling && (
        <button 
          className="scroll-to-bottom"
          onClick={() => {
            setIsAutoScrolling(true);
            listRef.current?.scrollToItem(messages.length - 1, 'end');
          }}
          aria-label="Scroll to newest messages"
        >
          ↓ New messages
        </button>
      )}
    </div>
  );
};
```

### Advanced Testing Patterns

#### Property-Based Testing for React Components

```jsx
// Property-based testing with fast-check
import fc from 'fast-check';

describe('Message Component Property Tests', () => {
  it('should always render valid HTML structure', () => {
    fc.assert(fc.property(
      fc.record({
        id: fc.string(),
        content: fc.string({ minLength: 1, maxLength: 4000 }),
        creatorId: fc.string(),
        createdAt: fc.date().map(d => d.toISOString()),
        updatedAt: fc.option(fc.date().map(d => d.toISOString()))
      }),
      fc.string(), // currentUserId
      (message, currentUserId) => {
        const { container } = render(
          <Message message={message} currentUserId={currentUserId} />
        );
        
        // Property: Should always have a message container
        expect(container.querySelector('[data-testid^="message-"]')).toBeInTheDocument();
        
        // Property: Content should be escaped and safe
        expect(container.innerHTML).not.toContain('<script');
        
        // Property: Should have proper ARIA attributes
        const messageElement = container.querySelector('[role="article"]');
        expect(messageElement).toBeInTheDocument();
        
        // Property: Timestamps should be valid
        const timeElement = container.querySelector('time');
        if (timeElement) {
          expect(timeElement.getAttribute('datetime')).toMatch(/^\d{4}-\d{2}-\d{2}T/);
        }
      }
    ));
  });

  it('should handle edge cases in message content', () => {
    fc.assert(fc.property(
      fc.oneof(
        fc.constant(''), // Empty string
        fc.string({ minLength: 4000, maxLength: 4000 }), // Max length
        fc.string().filter(s => s.includes('<script>')), // XSS attempt
        fc.string().filter(s => s.includes('\n'.repeat(100))), // Many newlines
        fc.unicodeString() // Unicode characters
      ),
      (content) => {
        const message = {
          id: 'test-id',
          content,
          creatorId: 'user-1',
          createdAt: new Date().toISOString()
        };
        
        // Should not throw errors regardless of content
        expect(() => {
          render(<Message message={message} currentUserId="user-2" />);
        }).not.toThrow();
      }
    ));
  });
});
```

#### Integration Testing with Mock Service Worker

```jsx
// Realistic API testing with MSW
import { setupServer } from 'msw/node';
import { rest } from 'msw';

const server = setupServer(
  rest.get('/api/messages', (req, res, ctx) => {
    const roomId = req.url.searchParams.get('roomId');
    const limit = parseInt(req.url.searchParams.get('limit') || '50');
    
    return res(
      ctx.json({
        messages: Array.from({ length: limit }, (_, i) => ({
          id: `msg-${i}`,
          content: `Test message ${i}`,
          creatorId: `user-${i % 3}`,
          roomId,
          createdAt: new Date(Date.now() - i * 60000).toISOString()
        }))
      })
    );
  }),
  
  rest.post('/api/messages', (req, res, ctx) => {
    return res(
      ctx.json({
        id: `msg-${Date.now()}`,
        ...req.body,
        createdAt: new Date().toISOString()
      })
    );
  })
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('ChatRoom Integration Tests', () => {
  it('should load and display messages from API', async () => {
    render(<ChatRoom roomId="room-1" />);
    
    // Wait for messages to load
    await waitFor(() => {
      expect(screen.getByText('Test message 0')).toBeInTheDocument();
    });
    
    // Should display multiple messages
    expect(screen.getAllByRole('article')).toHaveLength(50);
  });
  
  it('should send new message and update UI optimistically', async () => {
    const user = userEvent.setup();
    render(<ChatRoom roomId="room-1" />);
    
    // Wait for initial load
    await waitFor(() => {
      expect(screen.getByText('Test message 0')).toBeInTheDocument();
    });
    
    // Type and send message
    const input = screen.getByRole('textbox');
    await user.type(input, 'New test message');
    await user.click(screen.getByRole('button', { name: /send/i }));
    
    // Should appear immediately (optimistic update)
    expect(screen.getByText('New test message')).toBeInTheDocument();
    
    // Should show sending status
    expect(screen.getByText('Sending...')).toBeInTheDocument();
    
    // Wait for server confirmation
    await waitFor(() => {
      expect(screen.queryByText('Sending...')).not.toBeInTheDocument();
    });
  });
});
```

---

## Summary of Advanced Patterns

This comprehensive analysis provides the complete foundation for implementing Campfire's React frontend using the most advanced patterns and techniques available in the modern React ecosystem.

**Key Advanced Takeaways**:

1. **TDD as Design Tool** - Use Red-Green-Refactor to naturally arrive at idiomatic patterns
2. **Advanced Composition** - Leverage compound components and render props for maximum flexibility
3. **Sophisticated State Management** - Implement optimistic updates with rollback capabilities
4. **Performance Excellence** - Use dynamic virtualization and strategic memoization
5. **Comprehensive Testing** - Apply property-based testing and realistic API mocking
6. **Error Resilience** - Implement multi-level error boundaries with graceful degradation
7. **Accessibility First** - Ensure all patterns include proper ARIA attributes and semantic HTML
8. **Type Safety** - Use TypeScript to encode business logic in the type system

By combining these advanced patterns with the foundational React idioms, the Campfire frontend will deliver exceptional performance, maintainability, and user experience while following the highest standards of modern React development.