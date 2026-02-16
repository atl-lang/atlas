// WebSocket connection
let ws;
let reconnectAttempts = 0;
const maxReconnectAttempts = 10;
const reconnectDelay = 2000;

// Connection management
function connect() {
    const wsUrl = 'ws://' + window.location.host + '/ws';
    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
        console.log('WebSocket connected');
        updateConnectionStatus(true);
        reconnectAttempts = 0;
    };

    ws.onmessage = (event) => {
        const msg = JSON.parse(event.data);
        handleMessage(msg);
    };

    ws.onclose = () => {
        console.log('WebSocket disconnected');
        updateConnectionStatus(false);

        if (reconnectAttempts < maxReconnectAttempts) {
            setTimeout(() => {
                reconnectAttempts++;
                connect();
            }, reconnectDelay);
        }
    };

    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        updateConnectionStatus(false);
    };
}

// Update connection status indicator
function updateConnectionStatus(connected) {
    const dot = document.getElementById('connection-dot');
    const status = document.getElementById('connection-status');

    if (connected) {
        dot.className = 'w-3 h-3 rounded-full bg-green-500 animate-pulse';
        status.textContent = 'Connected';
        gsap.to('#status-indicator', {
            borderColor: 'rgba(0, 255, 65, 0.5)',
            boxShadow: '0 0 10px rgba(0, 255, 65, 0.3)',
            duration: 0.5
        });
    } else {
        dot.className = 'w-3 h-3 rounded-full bg-red-500 animate-pulse';
        status.textContent = reconnectAttempts < maxReconnectAttempts ? 'Reconnecting...' : 'Disconnected';
        gsap.to('#status-indicator', {
            borderColor: 'rgba(255, 59, 59, 0.5)',
            boxShadow: '0 0 10px rgba(255, 59, 59, 0.3)',
            duration: 0.5
        });
    }
}

// Handle incoming messages
function handleMessage(msg) {
    console.log('Received:', msg);

    switch (msg.type) {
        case 'stats_changed':
            updateStats(msg.payload);
            break;
        case 'phase_update':
            addUpdate('Phase Update', msg.payload.name + ' [' + msg.payload.status + ']', 'phase');
            break;
        case 'decision_added':
            addUpdate('Decision', msg.payload.title, 'decision');
            break;
        case 'feature_update':
            addUpdate('Feature Update', msg.payload.display_name + ' [' + msg.payload.status + ']', 'feature');
            break;
    }
}

// Update statistics with animations
function updateStats(stats) {
    // Animate number changes
    animateNumber('total-phases', stats.total_phases);
    animateNumber('completed-phases', stats.completed_phases);
    animateNumber('pending-phases', stats.pending_phases);
    animateNumber('total-decisions', stats.total_decisions);
    animateNumber('total-features', stats.total_features);

    // Update completion percentage
    const percent = stats.completion_rate.toFixed(1);
    document.getElementById('completion-percent').textContent = percent + '%';

    gsap.to('#completion-bar', {
        width: percent + '%',
        duration: 1,
        ease: 'power2.out'
    });

    // Update last update time
    const now = new Date();
    document.getElementById('last-update').textContent = now.toLocaleTimeString();

    // Pulse effect on stats panel
    gsap.fromTo('#stats-grid',
        { opacity: 0.7 },
        { opacity: 1, duration: 0.5 }
    );
}

// Animate number changes
function animateNumber(elementId, targetValue) {
    const element = document.getElementById(elementId);
    const currentValue = parseInt(element.textContent) || 0;

    if (currentValue !== targetValue) {
        gsap.to(element, {
            textContent: targetValue,
            duration: 1,
            ease: 'power2.out',
            snap: { textContent: 1 },
            onUpdate: function() {
                element.textContent = Math.round(element.textContent);
            }
        });

        // Glow effect on change
        gsap.fromTo(element,
            { textShadow: '0 0 20px rgba(0, 212, 255, 0.8)' },
            { textShadow: '0 0 0px rgba(0, 212, 255, 0)', duration: 1 }
        );
    }
}

// Add update to activity feed
function addUpdate(type, message, category) {
    const feed = document.getElementById('updates-feed');

    // Remove loading message if present
    if (feed.querySelector('.animate-pulse-slow')) {
        feed.innerHTML = '';
    }

    // Create update item
    const item = document.createElement('div');
    item.className = 'update-item bg-gray-800/50 rounded p-3 opacity-0';

    const timestamp = new Date().toLocaleTimeString();
    const icons = {
        phase: '📋',
        decision: '💡',
        feature: '⚡'
    };

    item.innerHTML = `
        <div class="flex justify-between items-start">
            <div class="flex-1">
                <div class="text-sm font-bold text-cyber-border">${icons[category] || '•'} ${type}</div>
                <div class="text-xs text-gray-300 mt-1">${message}</div>
            </div>
            <div class="text-xs text-gray-500 ml-2">${timestamp}</div>
        </div>
    `;

    // Insert at top
    feed.insertBefore(item, feed.firstChild);

    // Animate in
    gsap.fromTo(item,
        { opacity: 0, x: -20 },
        { opacity: 1, x: 0, duration: 0.5, ease: 'power2.out' }
    );

    // Keep only last 30 updates
    while (feed.children.length > 30) {
        const lastItem = feed.lastChild;
        gsap.to(lastItem, {
            opacity: 0,
            x: 20,
            duration: 0.3,
            onComplete: () => lastItem.remove()
        });
    }
}

// Initialize
connect();

// Send heartbeat every 30 seconds
setInterval(() => {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'ping' }));
    }
}, 30000);

// Add some initial animation
window.addEventListener('DOMContentLoaded', () => {
    gsap.from('header', { opacity: 0, y: -20, duration: 0.8, ease: 'power2.out' });
    gsap.from('.cyber-border', {
        opacity: 0,
        y: 20,
        duration: 0.6,
        stagger: 0.1,
        ease: 'power2.out'
    });
});
