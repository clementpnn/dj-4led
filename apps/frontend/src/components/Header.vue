<!-- src/components/Header.vue -->
<template>
    <header class="header">
        <div class="logo">
            <div class="logo-icon">🎵</div>
            <h1>DJ-4LED</h1>
            <div class="logo-icon">💡</div>
        </div>
        <div class="header-controls">
            <div class="status-badge" :class="{ connected: isConnected }">
                <div class="status-dot"></div>
                {{ isConnected ? "Connected" : "Disconnected" }}
            </div>
            <button
                class="settings-button"
                @click="$emit('toggle-settings')"
                title="Configuration"
            >
                ⚙️
            </button>
        </div>
    </header>
</template>

<script setup>
defineProps({
    isConnected: {
        type: Boolean,
        default: false,
    },
});

defineEmits(["toggle-settings"]);
</script>

<style scoped>
.header {
    padding: 2rem;
    text-align: center;
    border-bottom: 1px solid #21262d;
    position: relative;
}

.logo {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    margin-bottom: 1.5rem;
}

.logo-icon {
    font-size: 2.5rem;
    opacity: 0.8;
}

.logo h1 {
    font-size: 3rem;
    font-weight: 600;
    margin: 0;
    color: #f0f6fc;
    letter-spacing: 1px;
}

.header-controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
}

.status-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1.5rem;
    background: #21262d;
    border: 1px solid #30363d;
    border-radius: 8px;
    font-weight: 500;
    font-size: 0.9rem;
}

.status-badge.connected {
    background: #0d4929;
    border-color: #1a7f37;
    color: #2ea043;
}

.status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #f85149;
}

.status-badge.connected .status-dot {
    background: #2ea043;
}

.settings-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    background: #21262d;
    border: 1px solid #30363d;
    border-radius: 8px;
    font-size: 1.5rem;
    cursor: pointer;
    transition: all 0.2s ease;
}

.settings-button:hover {
    background: #30363d;
    border-color: #484f58;
    transform: rotate(45deg);
}

.settings-button:active {
    transform: rotate(45deg) scale(0.95);
}

@media (max-width: 768px) {
    .logo h1 {
        font-size: 2.5rem;
    }

    .header-controls {
        flex-direction: column;
        gap: 0.5rem;
    }

    .settings-button {
        width: 40px;
        height: 40px;
        font-size: 1.25rem;
    }
}
</style>
