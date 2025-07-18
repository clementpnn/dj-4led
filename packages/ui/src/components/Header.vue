<template>
    <div class="header">
        <h1>DJ-4LED</h1>
        <div class="status">
            <span :class="isConnected ? 'connected' : 'disconnected'">
                {{ isConnected ? '● Connected' : '○ Disconnected' }}
            </span>
            <span v-if="isConnected && fps > 0" class="fps"> {{ fps }} FPS </span>
        </div>
    </div>
</template>

<script setup>
const props = defineProps({
    isConnected: {
        type: Boolean,
        default: false,
    },
    fps: {
        type: Number,
        default: 0,
    },
});
</script>

<style scoped>
.header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 2rem;
    background: rgba(22, 27, 34, 0.8);
    backdrop-filter: blur(10px);
    border-bottom: 1px solid #30363d;
    position: sticky;
    top: 0;
    z-index: 100;
}

.header h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.logo {
    font-size: 1.75rem;
    filter: drop-shadow(0 0 8px rgba(88, 166, 255, 0.4));
}

.status {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-size: 0.875rem;
}

.connected {
    color: #238636;
    font-weight: 500;
}

.disconnected {
    color: #f85149;
    font-weight: 500;
}

.fps {
    background: linear-gradient(45deg, #238636, #2ea043);
    color: white;
    padding: 0.25rem 0.5rem;
    border-radius: 12px;
    font-weight: 600;
    font-size: 0.75rem;
}

@media (max-width: 768px) {
    .header {
        flex-direction: column;
        gap: 1rem;
        padding: 1rem;
    }
}
</style>
