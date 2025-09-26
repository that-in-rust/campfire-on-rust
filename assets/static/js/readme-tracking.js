// Simple README Analytics Tracking
// Privacy-friendly tracking for deployment success metrics

(function() {
    'use strict';
    
    // Track install script copy/download events
    function trackInstallScriptInteraction() {
        try {
            // Create tracking pixel
            const img = new Image();
            img.src = 'https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/api/analytics/track/install-download';
            img.style.display = 'none';
            document.body.appendChild(img);
            
            // Remove after tracking
            setTimeout(() => {
                if (img.parentNode) {
                    img.parentNode.removeChild(img);
                }
            }, 1000);
        } catch (error) {
            // Fail silently for privacy
            console.log('Tracking failed:', error);
        }
    }
    
    // Track deploy button clicks
    function trackDeployButtonClick(source, deploymentType) {
        try {
            const img = new Image();
            const params = new URLSearchParams({
                source: source || 'readme',
                deployment_type: deploymentType || 'railway'
            });
            img.src = `https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/api/analytics/track/deploy-click?${params}`;
            img.style.display = 'none';
            document.body.appendChild(img);
            
            setTimeout(() => {
                if (img.parentNode) {
                    img.parentNode.removeChild(img);
                }
            }, 1000);
        } catch (error) {
            console.log('Tracking failed:', error);
        }
    }
    
    // Auto-track when page loads (for README views)
    document.addEventListener('DOMContentLoaded', function() {
        // Track install script code block interactions
        const codeBlocks = document.querySelectorAll('code, pre');
        codeBlocks.forEach(block => {
            if (block.textContent && block.textContent.includes('curl') && block.textContent.includes('install.sh')) {
                block.addEventListener('click', trackInstallScriptInteraction);
                block.addEventListener('copy', trackInstallScriptInteraction);
            }
        });
        
        // Track Railway deploy button clicks
        const deployButtons = document.querySelectorAll('a[href*="railway.app"]');
        deployButtons.forEach(button => {
            button.addEventListener('click', () => {
                trackDeployButtonClick('readme', 'railway');
            });
        });
        
        // Track copy-to-clipboard events for install commands
        document.addEventListener('copy', function(e) {
            const selection = window.getSelection().toString();
            if (selection.includes('curl') && selection.includes('install.sh')) {
                trackInstallScriptInteraction();
            }
        });
    });
    
    // Export functions for manual tracking
    window.CampfireTracking = {
        trackInstallScript: trackInstallScriptInteraction,
        trackDeployClick: trackDeployButtonClick
    };
})();