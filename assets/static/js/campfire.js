// Campfire Chat Interface JavaScript

class CampfireApp {
    constructor() {
        this.ws = null;
        this.currentRoomId = null;
        this.currentUser = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 1000;
        
        this.init();
    }
    
    async init() {
        try {
            // Get current user info
            await this.getCurrentUser();
            
            // Initialize WebSocket connection
            this.connectWebSocket();
            
            // Set up event listeners
            this.setupEventListeners();
            
            // Load initial data
            await this.loadRooms();
            
            console.log('Campfire app initialized');
        } catch (error) {
            console.error('Failed to initialize Campfire app:', error);
            this.showError('Failed to initialize application');
        }
    }
    
    async getCurrentUser() {
        try {
            const response = await fetch('/api/users/me');
            if (response.ok) {
                this.currentUser = await response.json();
                this.updateUserInfo();
            } else {
                // Redirect to login if not authenticated
                window.location.href = '/login';
            }
        } catch (error) {
            console.error('Failed to get current user:', error);
            throw error;
        }
    }
    
    updateUserInfo() {
        const userInfoElement = document.querySelector('.user-info');
        if (userInfoElement && this.currentUser) {
            userInfoElement.textContent = this.currentUser.email;
        }
    }
    
    connectWebSocket() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;
        
        this.ws = new WebSocket(wsUrl);
        
        this.ws.onopen = () => {
            console.log('WebSocket connected');
            this.reconnectAttempts = 0;
            this.updateConnectionStatus(true);
        };
        
        this.ws.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                this.handleWebSocketMessage(message);
            } catch (error) {
                console.error('Failed to parse WebSocket message:', error);
            }
        };
        
        this.ws.onclose = () => {
            console.log('WebSocket disconnected');
            this.updateConnectionStatus(false);
            this.scheduleReconnect();
        };
        
        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };
    }
    
    scheduleReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
            
            console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);
            
            setTimeout(() => {
                this.connectWebSocket();
            }, delay);
        } else {
            console.error('Max reconnection attempts reached');
            this.showError('Connection lost. Please refresh the page.');
        }
    }
    
    updateConnectionStatus(connected) {
        const statusElement = document.querySelector('.connection-status');
        if (statusElement) {
            statusElement.textContent = connected ? 'Connected' : 'Disconnected';
            statusElement.className = `connection-status ${connected ? 'connected' : 'disconnected'}`;
        }
    }
    
    handleWebSocketMessage(message) {
        switch (message.type) {
            case 'new_message':
                this.handleNewMessage(message.data);
                break;
            case 'user_joined':
                this.handleUserJoined(message.data);
                break;
            case 'user_left':
                this.handleUserLeft(message.data);
                break;
            case 'typing_start':
                this.handleTypingStart(message.data);
                break;
            case 'typing_stop':
                this.handleTypingStop(message.data);
                break;
            case 'sound_played':
                this.handleSoundPlayed(message.data);
                break;
            default:
                console.log('Unknown WebSocket message type:', message.type);
        }
    }
    
    handleNewMessage(messageData) {
        if (messageData.room_id === this.currentRoomId) {
            this.addMessageToChat(messageData);
            this.scrollToBottom();
        }
        
        // Update unread count for other rooms
        this.updateUnreadCount(messageData.room_id);
        
        // Play notification sound if not from current user
        if (messageData.creator_id !== this.currentUser?.id) {
            this.playNotificationSound();
        }
    }
    
    handleUserJoined(data) {
        this.addSystemMessage(`${data.user_name} joined the room`);
    }
    
    handleUserLeft(data) {
        this.addSystemMessage(`${data.user_name} left the room`);
    }
    
    handleTypingStart(data) {
        if (data.user_id !== this.currentUser?.id) {
            this.showTypingIndicator(data.user_name);
        }
    }
    
    handleTypingStop(data) {
        if (data.user_id !== this.currentUser?.id) {
            this.hideTypingIndicator(data.user_name);
        }
    }
    
    handleSoundPlayed(data) {
        this.playSound(data.sound_name);
        this.addSoundNotification(data.sound_name, data.user_name);
    }
    
    setupEventListeners() {
        // Message form submission
        const messageForm = document.querySelector('.composer-form');
        if (messageForm) {
            messageForm.addEventListener('submit', (e) => {
                e.preventDefault();
                this.sendMessage();
            });
        }
        
        // Message input typing detection
        const messageInput = document.querySelector('.composer-input');
        if (messageInput) {
            let typingTimer;
            
            messageInput.addEventListener('input', () => {
                this.sendTypingStart();
                
                clearTimeout(typingTimer);
                typingTimer = setTimeout(() => {
                    this.sendTypingStop();
                }, 1000);
            });
            
            messageInput.addEventListener('keydown', (e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    this.sendMessage();
                }
            });
        }
        
        // Room selection
        document.addEventListener('click', (e) => {
            const roomItem = e.target.closest('.room-item');
            if (roomItem) {
                e.preventDefault();
                const roomId = roomItem.dataset.roomId;
                this.selectRoom(roomId);
            }
        });
    }
    
    async loadRooms() {
        try {
            const response = await fetch('/api/rooms');
            if (response.ok) {
                const rooms = await response.json();
                this.renderRooms(rooms);
                
                // Select first room if none selected
                if (rooms.length > 0 && !this.currentRoomId) {
                    this.selectRoom(rooms[0].id);
                }
            } else {
                throw new Error('Failed to load rooms');
            }
        } catch (error) {
            console.error('Failed to load rooms:', error);
            this.showError('Failed to load rooms');
        }
    }
    
    renderRooms(rooms) {
        const roomsList = document.querySelector('.rooms-list');
        if (!roomsList) return;
        
        roomsList.innerHTML = rooms.map(room => `
            <a href="#" class="room-item" data-room-id="${room.id}">
                <img src="/static/images/messages.svg" alt="" class="room-icon">
                <span class="room-name">${this.escapeHtml(room.name)}</span>
                <span class="unread-count" style="display: none;">0</span>
            </a>
        `).join('');
    }
    
    async selectRoom(roomId) {
        if (this.currentRoomId === roomId) return;
        
        this.currentRoomId = roomId;
        
        // Update UI
        this.updateActiveRoom();
        this.clearMessages();
        this.showLoading();
        
        try {
            // Load room messages
            await this.loadRoomMessages(roomId);
            
            // Update room info
            await this.loadRoomInfo(roomId);
            
        } catch (error) {
            console.error('Failed to select room:', error);
            this.showError('Failed to load room');
        }
    }
    
    updateActiveRoom() {
        document.querySelectorAll('.room-item').forEach(item => {
            item.classList.toggle('active', item.dataset.roomId === this.currentRoomId);
        });
    }
    
    async loadRoomMessages(roomId) {
        try {
            const response = await fetch(`/api/rooms/${roomId}/messages?limit=50`);
            if (response.ok) {
                const messages = await response.json();
                this.renderMessages(messages);
                this.scrollToBottom();
            } else {
                throw new Error('Failed to load messages');
            }
        } catch (error) {
            console.error('Failed to load room messages:', error);
            throw error;
        }
    }
    
    async loadRoomInfo(roomId) {
        try {
            const response = await fetch(`/api/rooms/${roomId}`);
            if (response.ok) {
                const room = await response.json();
                this.updateRoomHeader(room);
            }
        } catch (error) {
            console.error('Failed to load room info:', error);
        }
    }
    
    updateRoomHeader(room) {
        const titleElement = document.querySelector('.room-title');
        const topicElement = document.querySelector('.room-topic');
        
        if (titleElement) {
            titleElement.textContent = room.name;
        }
        
        if (topicElement) {
            topicElement.textContent = room.topic || '';
            topicElement.style.display = room.topic ? 'block' : 'none';
        }
    }
    
    renderMessages(messages) {
        const container = document.querySelector('.messages-container');
        if (!container) return;
        
        container.innerHTML = messages.map(message => this.renderMessage(message)).join('');
    }
    
    renderMessage(message) {
        const time = new Date(message.created_at).toLocaleTimeString([], {
            hour: '2-digit',
            minute: '2-digit'
        });
        
        const initials = message.creator_name
            .split(' ')
            .map(name => name[0])
            .join('')
            .toUpperCase()
            .slice(0, 2);
        
        return `
            <div class="message" data-message-id="${message.id}">
                <div class="message-avatar">${initials}</div>
                <div class="message-content">
                    <div class="message-header">
                        <span class="message-author">${this.escapeHtml(message.creator_name)}</span>
                        <span class="message-time">${time}</span>
                    </div>
                    <div class="message-body">${this.formatMessageContent(message.content)}</div>
                </div>
            </div>
        `;
    }
    
    formatMessageContent(content) {
        // Basic HTML sanitization and formatting
        let formatted = this.escapeHtml(content);
        
        // Convert URLs to links
        formatted = formatted.replace(
            /(https?:\/\/[^\s]+)/g,
            '<a href="$1" target="_blank" rel="noopener noreferrer">$1</a>'
        );
        
        // Convert @mentions
        formatted = formatted.replace(
            /@(\w+)/g,
            '<span class="mention">@$1</span>'
        );
        
        // Convert line breaks
        formatted = formatted.replace(/\n/g, '<br>');
        
        return formatted;
    }
    
    addMessageToChat(messageData) {
        const container = document.querySelector('.messages-container');
        if (!container) return;
        
        const messageHtml = this.renderMessage(messageData);
        container.insertAdjacentHTML('beforeend', messageHtml);
    }
    
    addSystemMessage(text) {
        const container = document.querySelector('.messages-container');
        if (!container) return;
        
        const messageHtml = `
            <div class="message system">
                <div class="message-content">${this.escapeHtml(text)}</div>
            </div>
        `;
        
        container.insertAdjacentHTML('beforeend', messageHtml);
        this.scrollToBottom();
    }
    
    addSoundNotification(soundName, userName) {
        const container = document.querySelector('.messages-container');
        if (!container) return;
        
        const messageHtml = `
            <div class="message system">
                <div class="message-content">
                    <span class="sound-notification">🔊 ${this.escapeHtml(userName)} played "${soundName}"</span>
                </div>
            </div>
        `;
        
        container.insertAdjacentHTML('beforeend', messageHtml);
        this.scrollToBottom();
    }
    
    async sendMessage() {
        const input = document.querySelector('.composer-input');
        if (!input || !this.currentRoomId) return;
        
        const content = input.value.trim();
        if (!content) return;
        
        try {
            const response = await fetch(`/api/rooms/${this.currentRoomId}/messages`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    content,
                    client_message_id: this.generateUUID()
                })
            });
            
            if (response.ok) {
                input.value = '';
                this.sendTypingStop();
            } else {
                throw new Error('Failed to send message');
            }
        } catch (error) {
            console.error('Failed to send message:', error);
            this.showError('Failed to send message');
        }
    }
    
    sendTypingStart() {
        if (this.ws && this.ws.readyState === WebSocket.OPEN && this.currentRoomId) {
            this.ws.send(JSON.stringify({
                type: 'typing_start',
                room_id: this.currentRoomId
            }));
        }
    }
    
    sendTypingStop() {
        if (this.ws && this.ws.readyState === WebSocket.OPEN && this.currentRoomId) {
            this.ws.send(JSON.stringify({
                type: 'typing_stop',
                room_id: this.currentRoomId
            }));
        }
    }
    
    showTypingIndicator(userName) {
        let indicator = document.querySelector('.typing-indicator');
        if (!indicator) {
            indicator = document.createElement('div');
            indicator.className = 'typing-indicator';
            document.querySelector('.messages-container').appendChild(indicator);
        }
        
        indicator.innerHTML = `${this.escapeHtml(userName)} is typing<span class="typing-dots">...</span>`;
        this.scrollToBottom();
    }
    
    hideTypingIndicator(userName) {
        const indicator = document.querySelector('.typing-indicator');
        if (indicator) {
            indicator.remove();
        }
    }
    
    playSound(soundName) {
        // Create and play audio element
        const audio = new Audio(`/api/sounds/${soundName}`);
        audio.volume = 0.5;
        audio.play().catch(error => {
            console.log('Could not play sound:', error);
        });
    }
    
    playNotificationSound() {
        // Play a subtle notification sound
        const audio = new Audio('/api/sounds/bell');
        audio.volume = 0.3;
        audio.play().catch(error => {
            console.log('Could not play notification sound:', error);
        });
    }
    
    updateUnreadCount(roomId) {
        if (roomId === this.currentRoomId) return;
        
        const roomItem = document.querySelector(`[data-room-id="${roomId}"]`);
        if (roomItem) {
            const unreadElement = roomItem.querySelector('.unread-count');
            if (unreadElement) {
                const current = parseInt(unreadElement.textContent) || 0;
                unreadElement.textContent = current + 1;
                unreadElement.style.display = 'inline-block';
            }
        }
    }
    
    clearMessages() {
        const container = document.querySelector('.messages-container');
        if (container) {
            container.innerHTML = '';
        }
    }
    
    showLoading() {
        const container = document.querySelector('.messages-container');
        if (container) {
            container.innerHTML = '<div class="loading"><div class="spinner"></div>Loading messages...</div>';
        }
    }
    
    scrollToBottom() {
        const container = document.querySelector('.messages-container');
        if (container) {
            container.scrollTop = container.scrollHeight;
        }
    }
    
    showError(message) {
        // Simple error display - in production, use a proper notification system
        console.error(message);
        alert(message);
    }
    
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    
    generateUUID() {
        return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
            const r = Math.random() * 16 | 0;
            const v = c == 'x' ? r : (r & 0x3 | 0x8);
            return v.toString(16);
        });
    }
}

// Initialize the app when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.campfire = new CampfireApp();
});

// Service Worker registration for PWA support
if ('serviceWorker' in navigator) {
    window.addEventListener('load', () => {
        navigator.serviceWorker.register('/static/js/sw.js')
            .then(registration => {
                console.log('SW registered: ', registration);
            })
            .catch(registrationError => {
                console.log('SW registration failed: ', registrationError);
            });
    });
}