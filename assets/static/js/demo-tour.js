/**
 * Campfire Demo Tour System
 * Provides guided tour and feature highlighting for demo users
 * Requirement 10.4: Guided tour highlighting @mentions, /play sounds, search, and real-time capabilities
 */

class CampfireDemoTour {
    constructor() {
        this.currentStep = 0;
        this.tourSteps = [];
        this.sessionId = null;
        this.userRole = null;
        this.tourOverlay = null;
        this.isActive = false;
        
        // Initialize tour if in demo mode
        this.init();
    }
    
    async init() {
        // Check if we're in demo mode and should show tour
        if (this.isDemoMode() && !this.isTourCompleted()) {
            await this.loadTourSteps();
            this.createTourOverlay();
            this.startTour();
        }
        
        // Set up event listeners for tour interactions
        this.setupEventListeners();
    }
    
    isDemoMode() {
        // Check if current user is a demo user
        const userEmail = this.getCurrentUserEmail();
        return userEmail && userEmail.includes('@campfire.demo');
    }
    
    getCurrentUserEmail() {
        // Extract user email from page context or session
        const userInfo = document.querySelector('[data-user-email]');
        return userInfo ? userInfo.dataset.userEmail : null;
    }
    
    getUserRole() {
        // Extract user role from page context
        const userInfo = document.querySelector('[data-user-role]');
        return userInfo ? userInfo.dataset.userRole : 'Member';
    }
    
    isTourCompleted() {
        // Check if tour was already completed in this session
        return localStorage.getItem('campfire_tour_completed') === 'true';
    }
    
    async loadTourSteps() {
        try {
            this.userRole = this.getUserRole();
            const response = await fetch(`/api/demo/tour-steps?role=${encodeURIComponent(this.userRole)}`);
            const data = await response.json();
            
            if (data.success) {
                this.tourSteps = data.tour_steps;
            }
        } catch (error) {
            console.warn('Failed to load tour steps:', error);
        }
    }
    
    createTourOverlay() {
        // Create tour overlay container
        this.tourOverlay = document.createElement('div');
        this.tourOverlay.className = 'demo-tour-overlay';
        this.tourOverlay.innerHTML = `
            <div class="tour-backdrop"></div>
            <div class="tour-popup">
                <div class="tour-header">
                    <h3 class="tour-title"></h3>
                    <button class="tour-close" aria-label="Close tour">&times;</button>
                </div>
                <div class="tour-content">
                    <p class="tour-description"></p>
                    <div class="tour-action-hint"></div>
                </div>
                <div class="tour-footer">
                    <div class="tour-progress">
                        <span class="tour-step-counter"></span>
                        <div class="tour-progress-bar">
                            <div class="tour-progress-fill"></div>
                        </div>
                    </div>
                    <div class="tour-buttons">
                        <button class="tour-btn tour-btn-skip">Skip Tour</button>
                        <button class="tour-btn tour-btn-prev">Previous</button>
                        <button class="tour-btn tour-btn-next">Next</button>
                    </div>
                </div>
            </div>
        `;
        
        // Add tour styles
        this.addTourStyles();
        
        document.body.appendChild(this.tourOverlay);
        
        // Set up tour overlay event listeners
        this.setupTourEventListeners();
    }
    
    addTourStyles() {
        if (document.getElementById('demo-tour-styles')) return;
        
        const styles = document.createElement('style');
        styles.id = 'demo-tour-styles';
        styles.textContent = `
            .demo-tour-overlay {
                position: fixed;
                top: 0;
                left: 0;
                width: 100%;
                height: 100%;
                z-index: 10000;
                pointer-events: none;
            }
            
            .tour-backdrop {
                position: absolute;
                top: 0;
                left: 0;
                width: 100%;
                height: 100%;
                background: rgba(0, 0, 0, 0.7);
                pointer-events: auto;
            }
            
            .tour-popup {
                position: absolute;
                background: white;
                border-radius: 12px;
                box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
                max-width: 400px;
                min-width: 320px;
                pointer-events: auto;
                z-index: 10001;
            }
            
            .tour-header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 1.5rem 1.5rem 0;
            }
            
            .tour-title {
                margin: 0;
                font-size: 1.25rem;
                font-weight: 600;
                color: #2c3e50;
            }
            
            .tour-close {
                background: none;
                border: none;
                font-size: 1.5rem;
                cursor: pointer;
                color: #7f8c8d;
                padding: 0;
                width: 24px;
                height: 24px;
                display: flex;
                align-items: center;
                justify-content: center;
            }
            
            .tour-close:hover {
                color: #e74c3c;
            }
            
            .tour-content {
                padding: 1rem 1.5rem;
            }
            
            .tour-description {
                margin: 0 0 1rem;
                line-height: 1.6;
                color: #555;
            }
            
            .tour-action-hint {
                background: #e8f4fd;
                border: 1px solid #3498db;
                border-radius: 6px;
                padding: 0.75rem;
                font-size: 0.9rem;
                color: #2980b9;
                display: none;
            }
            
            .tour-action-hint.visible {
                display: block;
            }
            
            .tour-footer {
                padding: 0 1.5rem 1.5rem;
            }
            
            .tour-progress {
                display: flex;
                align-items: center;
                gap: 1rem;
                margin-bottom: 1rem;
            }
            
            .tour-step-counter {
                font-size: 0.85rem;
                color: #7f8c8d;
                white-space: nowrap;
            }
            
            .tour-progress-bar {
                flex: 1;
                height: 4px;
                background: #ecf0f1;
                border-radius: 2px;
                overflow: hidden;
            }
            
            .tour-progress-fill {
                height: 100%;
                background: #3498db;
                border-radius: 2px;
                transition: width 0.3s ease;
            }
            
            .tour-buttons {
                display: flex;
                gap: 0.5rem;
                justify-content: flex-end;
            }
            
            .tour-btn {
                padding: 0.5rem 1rem;
                border: 1px solid #bdc3c7;
                border-radius: 6px;
                background: white;
                color: #2c3e50;
                cursor: pointer;
                font-size: 0.9rem;
                transition: all 0.2s ease;
            }
            
            .tour-btn:hover {
                background: #ecf0f1;
            }
            
            .tour-btn-next {
                background: #3498db;
                color: white;
                border-color: #3498db;
            }
            
            .tour-btn-next:hover {
                background: #2980b9;
            }
            
            .tour-btn:disabled {
                opacity: 0.5;
                cursor: not-allowed;
            }
            
            .tour-highlight {
                position: relative;
                z-index: 9999;
                box-shadow: 0 0 0 4px rgba(52, 152, 219, 0.5), 0 0 20px rgba(52, 152, 219, 0.3);
                border-radius: 4px;
                animation: tourPulse 2s infinite;
            }
            
            @keyframes tourPulse {
                0%, 100% { box-shadow: 0 0 0 4px rgba(52, 152, 219, 0.5), 0 0 20px rgba(52, 152, 219, 0.3); }
                50% { box-shadow: 0 0 0 8px rgba(52, 152, 219, 0.3), 0 0 30px rgba(52, 152, 219, 0.5); }
            }
            
            .tour-spotlight {
                position: absolute;
                border: 2px solid #3498db;
                border-radius: 8px;
                background: rgba(52, 152, 219, 0.1);
                pointer-events: none;
                z-index: 9998;
                animation: tourSpotlight 1s ease-out;
            }
            
            @keyframes tourSpotlight {
                from { transform: scale(0.8); opacity: 0; }
                to { transform: scale(1); opacity: 1; }
            }
        `;
        
        document.head.appendChild(styles);
    }
    
    setupTourEventListeners() {
        if (!this.tourOverlay) return;
        
        // Close tour
        this.tourOverlay.querySelector('.tour-close').addEventListener('click', () => {
            this.endTour();
        });
        
        // Skip tour
        this.tourOverlay.querySelector('.tour-btn-skip').addEventListener('click', () => {
            this.endTour();
        });
        
        // Previous step
        this.tourOverlay.querySelector('.tour-btn-prev').addEventListener('click', () => {
            this.previousStep();
        });
        
        // Next step
        this.tourOverlay.querySelector('.tour-btn-next').addEventListener('click', () => {
            this.nextStep();
        });
        
        // Close on backdrop click
        this.tourOverlay.querySelector('.tour-backdrop').addEventListener('click', () => {
            this.endTour();
        });
    }
    
    setupEventListeners() {
        // Listen for demo-specific interactions
        document.addEventListener('click', (e) => {
            if (!this.isActive) return;
            
            // Track feature exploration
            this.trackFeatureInteraction(e.target);
        });
        
        // Listen for message sending
        document.addEventListener('submit', (e) => {
            if (e.target.matches('.message-form')) {
                this.trackFeature('message_sent');
            }
        });
        
        // Listen for search usage
        document.addEventListener('input', (e) => {
            if (e.target.matches('.search-input')) {
                this.trackFeature('search_used');
            }
        });
        
        // Listen for room switching
        document.addEventListener('click', (e) => {
            if (e.target.matches('.room-item') || e.target.closest('.room-item')) {
                this.trackFeature('room_switched');
            }
        });
    }
    
    startTour() {
        if (this.tourSteps.length === 0) return;
        
        this.isActive = true;
        this.currentStep = 0;
        this.showStep(this.currentStep);
        
        // Start simulation session
        this.startSimulationSession();
    }
    
    async startSimulationSession() {
        try {
            const response = await fetch('/api/demo/start-session', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    user_email: this.getCurrentUserEmail(),
                    browser_tab_id: this.generateTabId()
                })
            });
            
            const data = await response.json();
            if (data.success) {
                this.sessionId = data.session.session_id;
            }
        } catch (error) {
            console.warn('Failed to start simulation session:', error);
        }
    }
    
    generateTabId() {
        // Generate unique tab ID for multi-tab simulation
        return `tab_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    }
    
    showStep(stepIndex) {
        if (stepIndex < 0 || stepIndex >= this.tourSteps.length) return;
        
        const step = this.tourSteps[stepIndex];
        
        // Update popup content
        this.tourOverlay.querySelector('.tour-title').textContent = step.title;
        this.tourOverlay.querySelector('.tour-description').textContent = step.description;
        
        // Update action hint
        const actionHint = this.tourOverlay.querySelector('.tour-action-hint');
        if (step.action_required) {
            actionHint.textContent = `Action: ${step.action_required}`;
            actionHint.classList.add('visible');
        } else {
            actionHint.classList.remove('visible');
        }
        
        // Update progress
        const progress = ((stepIndex + 1) / this.tourSteps.length) * 100;
        this.tourOverlay.querySelector('.tour-progress-fill').style.width = `${progress}%`;
        this.tourOverlay.querySelector('.tour-step-counter').textContent = 
            `Step ${stepIndex + 1} of ${this.tourSteps.length}`;
        
        // Update button states
        this.tourOverlay.querySelector('.tour-btn-prev').disabled = stepIndex === 0;
        const nextBtn = this.tourOverlay.querySelector('.tour-btn-next');
        nextBtn.textContent = stepIndex === this.tourSteps.length - 1 ? 'Finish' : 'Next';
        
        // Position popup and highlight target
        this.positionPopupAndHighlight(step);
        
        // Mark step as completed
        this.completeTourStep(step.step_id);
    }
    
    positionPopupAndHighlight(step) {
        const targetElement = document.querySelector(step.target_element);
        
        if (targetElement) {
            // Highlight target element
            this.clearHighlights();
            
            if (step.highlight_type === 'highlight') {
                targetElement.classList.add('tour-highlight');
            } else if (step.highlight_type === 'spotlight') {
                this.createSpotlight(targetElement);
            }
            
            // Position popup near target
            this.positionPopup(targetElement);
            
            // Scroll target into view
            targetElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
        } else {
            // Center popup if no target found
            this.centerPopup();
        }
    }
    
    positionPopup(targetElement) {
        const popup = this.tourOverlay.querySelector('.tour-popup');
        const rect = targetElement.getBoundingClientRect();
        const popupRect = popup.getBoundingClientRect();
        
        let top = rect.bottom + 20;
        let left = rect.left + (rect.width / 2) - (popupRect.width / 2);
        
        // Adjust if popup would go off screen
        if (left < 20) left = 20;
        if (left + popupRect.width > window.innerWidth - 20) {
            left = window.innerWidth - popupRect.width - 20;
        }
        
        if (top + popupRect.height > window.innerHeight - 20) {
            top = rect.top - popupRect.height - 20;
        }
        
        popup.style.top = `${top}px`;
        popup.style.left = `${left}px`;
    }
    
    centerPopup() {
        const popup = this.tourOverlay.querySelector('.tour-popup');
        popup.style.top = '50%';
        popup.style.left = '50%';
        popup.style.transform = 'translate(-50%, -50%)';
    }
    
    createSpotlight(targetElement) {
        const rect = targetElement.getBoundingClientRect();
        const spotlight = document.createElement('div');
        spotlight.className = 'tour-spotlight';
        spotlight.style.top = `${rect.top - 10}px`;
        spotlight.style.left = `${rect.left - 10}px`;
        spotlight.style.width = `${rect.width + 20}px`;
        spotlight.style.height = `${rect.height + 20}px`;
        
        document.body.appendChild(spotlight);
    }
    
    clearHighlights() {
        // Remove highlight classes
        document.querySelectorAll('.tour-highlight').forEach(el => {
            el.classList.remove('tour-highlight');
        });
        
        // Remove spotlight elements
        document.querySelectorAll('.tour-spotlight').forEach(el => {
            el.remove();
        });
    }
    
    nextStep() {
        if (this.currentStep < this.tourSteps.length - 1) {
            this.currentStep++;
            this.showStep(this.currentStep);
        } else {
            this.completeTour();
        }
    }
    
    previousStep() {
        if (this.currentStep > 0) {
            this.currentStep--;
            this.showStep(this.currentStep);
        }
    }
    
    async completeTourStep(stepId) {
        if (!this.sessionId) return;
        
        try {
            await fetch('/api/demo/complete-tour-step', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    session_id: this.sessionId,
                    step_id: stepId
                })
            });
        } catch (error) {
            console.warn('Failed to complete tour step:', error);
        }
    }
    
    completeTour() {
        localStorage.setItem('campfire_tour_completed', 'true');
        this.endTour();
        
        // Show completion message
        this.showCompletionMessage();
    }
    
    endTour() {
        this.isActive = false;
        this.clearHighlights();
        
        if (this.tourOverlay) {
            this.tourOverlay.remove();
            this.tourOverlay = null;
        }
    }
    
    showCompletionMessage() {
        // Create a simple completion notification
        const notification = document.createElement('div');
        notification.className = 'tour-completion-notification';
        notification.innerHTML = `
            <div class="notification-content">
                <h3>🎉 Tour Completed!</h3>
                <p>You've successfully explored Campfire's key features. Try opening multiple tabs to experience real-time collaboration!</p>
                <button class="notification-close">Got it!</button>
            </div>
        `;
        
        // Add notification styles
        const styles = `
            .tour-completion-notification {
                position: fixed;
                top: 20px;
                right: 20px;
                background: white;
                border-radius: 8px;
                box-shadow: 0 10px 30px rgba(0,0,0,0.2);
                z-index: 10000;
                max-width: 300px;
                animation: slideInRight 0.3s ease;
            }
            
            .notification-content {
                padding: 1.5rem;
            }
            
            .notification-content h3 {
                margin: 0 0 0.5rem;
                color: #27ae60;
            }
            
            .notification-content p {
                margin: 0 0 1rem;
                font-size: 0.9rem;
                line-height: 1.4;
            }
            
            .notification-close {
                background: #3498db;
                color: white;
                border: none;
                padding: 0.5rem 1rem;
                border-radius: 4px;
                cursor: pointer;
                font-size: 0.9rem;
            }
            
            @keyframes slideInRight {
                from { transform: translateX(100%); opacity: 0; }
                to { transform: translateX(0); opacity: 1; }
            }
        `;
        
        const styleSheet = document.createElement('style');
        styleSheet.textContent = styles;
        document.head.appendChild(styleSheet);
        
        document.body.appendChild(notification);
        
        // Auto-remove after 5 seconds or on click
        const removeNotification = () => {
            notification.remove();
            styleSheet.remove();
        };
        
        notification.querySelector('.notification-close').addEventListener('click', removeNotification);
        setTimeout(removeNotification, 5000);
    }
    
    trackFeatureInteraction(element) {
        // Track various feature interactions for analytics
        const features = [];
        
        if (element.matches('.mention') || element.textContent.includes('@')) {
            features.push('mention_interaction');
        }
        
        if (element.matches('.sound-command') || element.textContent.includes('/play')) {
            features.push('sound_command');
        }
        
        if (element.matches('.search-input, .search-button')) {
            features.push('search_interaction');
        }
        
        if (element.matches('.room-item')) {
            features.push('room_navigation');
        }
        
        if (features.length > 0) {
            this.trackFeatures(features);
        }
    }
    
    async trackFeature(feature) {
        this.trackFeatures([feature]);
    }
    
    async trackFeatures(features) {
        if (!this.sessionId || features.length === 0) return;
        
        try {
            await fetch('/api/demo/update-session', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    session_id: this.sessionId,
                    features_explored: features
                })
            });
        } catch (error) {
            console.warn('Failed to track features:', error);
        }
    }
}

// Initialize tour when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.campfireTour = new CampfireDemoTour();
});

// Export for manual control
window.CampfireDemoTour = CampfireDemoTour;