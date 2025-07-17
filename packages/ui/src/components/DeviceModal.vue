<!-- src/components/DeviceModal.vue -->
<template>
    <Teleport to="body">
        <div v-if="visible" class="modal-overlay" @click="handleOverlayClick">
            <div class="modal-container">
                <div class="modal-header">
                    <h2 class="modal-title">
                        {{ isEditing ? '⚙️ Modifier la valise' : '➕ Ajouter une valise' }}
                    </h2>
                    <button class="close-btn" @click="handleClose">❌</button>
                </div>

                <form @submit.prevent="handleSubmit" class="modal-form">
                    <!-- Device Name -->
                    <div class="form-group">
                        <label for="device-name" class="form-label">
                            🏷️ Nom de la valise
                        </label>
                        <input
                            id="device-name"
                            v-model="formData.name"
                            type="text"
                            class="form-input"
                            placeholder="Ex: Valise Scène, Valise Bar..."
                            :class="{ 'error': errors.name }"
                            required
                        />
                        <div v-if="errors.name" class="error-message">{{ errors.name }}</div>
                    </div>

                    <!-- IP Address -->
                    <div class="form-group">
                        <label for="device-ip" class="form-label">
                            🌐 Adresse IP
                        </label>
                        <input
                            id="device-ip"
                            v-model="formData.ipAddress"
                            type="text"
                            class="form-input"
                            placeholder="192.168.1.100"
                            :class="{ 'error': errors.ipAddress }"
                            @input="validateIP"
                            required
                        />
                        <div v-if="errors.ipAddress" class="error-message">{{ errors.ipAddress }}</div>
                        <div class="form-hint">
                            Format: xxx.xxx.xxx.xxx (ex: 192.168.1.100)
                        </div>
                    </div>

                    <!-- Port -->
                    <div class="form-group">
                        <label for="device-port" class="form-label">
                            🔌 Port
                        </label>
                        <input
                            id="device-port"
                            v-model.number="formData.port"
                            type="number"
                            class="form-input"
                            placeholder="8081"
                            :class="{ 'error': errors.port }"
                            min="1"
                            max="65535"
                            required
                        />
                        <div v-if="errors.port" class="error-message">{{ errors.port }}</div>
                        <div class="form-hint">
                            Port par défaut: 8081 (1-65535)
                        </div>
                    </div>

                    <!-- IP Suggestions -->
                    <div v-if="showSuggestions && ipSuggestions.length > 0" class="suggestions-section">
                        <h4 class="suggestions-title">💡 Suggestions d'IP</h4>
                        <div class="suggestions-grid">
                            <button
                                v-for="suggestion in ipSuggestions"
                                :key="suggestion.ip"
                                type="button"
                                class="suggestion-btn"
                                @click="applySuggestion(suggestion)"
                            >
                                <div class="suggestion-ip">{{ suggestion.ip }}</div>
                                <div class="suggestion-label">{{ suggestion.label }}</div>
                            </button>
                        </div>
                    </div>

                    <!-- Connection Test -->
                    <div v-if="formData.ipAddress && formData.port" class="test-section">
                        <button
                            type="button"
                            class="btn btn-info test-btn"
                            @click="testConnection"
                            :disabled="testing || !isValidForm"
                        >
                            <span v-if="testing">⏳ Test en cours...</span>
                            <span v-else>🧪 Tester la connexion</span>
                        </button>

                        <div v-if="testResult" class="test-result" :class="testResult.type">
                            {{ testResult.message }}
                        </div>
                    </div>

                    <!-- Form Actions -->
                    <div class="form-actions">
                        <button type="button" class="btn btn-secondary" @click="handleClose">
                            ❌ Annuler
                        </button>
                        <button
                            type="submit"
                            class="btn btn-primary"
                            :disabled="!isValidForm || submitting"
                        >
                            <span v-if="submitting">⏳ {{ isEditing ? 'Modification...' : 'Ajout...' }}</span>
                            <span v-else>{{ isEditing ? '✅ Modifier' : '➕ Ajouter' }}</span>
                        </button>
                    </div>
                </form>
            </div>
        </div>
    </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';

// Types
interface Device {
    id?: string;
    name: string;
    ipAddress: string;
    port: number;
}

interface TestResult {
    type: 'success' | 'error' | 'warning';
    message: string;
}

interface IPSuggestion {
    ip: string;
    label: string;
}

// Props
interface Props {
    visible: boolean;
    device?: Device;
}

const props = withDefaults(defineProps<Props>(), {
    device: undefined
});

// Emits
interface Emits {
    close: [];
    save: [device: Device];
    test: [device: Device];
}

const emit = defineEmits<Emits>();

// Form data
const formData = ref<Device>({
    name: '',
    ipAddress: '',
    port: 8081
});

const errors = ref<Record<string, string>>({});
const testing = ref(false);
const submitting = ref(false);
const testResult = ref<TestResult | null>(null);
const showSuggestions = ref(true);

// Computed
const isEditing = computed(() => !!props.device?.id);

const isValidForm = computed(() => {
    return formData.value.name.trim() !== '' &&
           isValidIP(formData.value.ipAddress) &&
           formData.value.port >= 1 &&
           formData.value.port <= 65535 &&
           Object.keys(errors.value).length === 0;
});

const ipSuggestions = computed((): IPSuggestion[] => {
    const suggestions: IPSuggestion[] = [
        { ip: '192.168.1.45', label: 'Valise 1 (config)' },
        { ip: '192.168.1.46', label: 'Valise 2 (config)' },
        { ip: '192.168.1.47', label: 'Valise 3 (config)' },
        { ip: '192.168.1.48', label: 'Valise 4 (config)' },
        { ip: '192.168.1.100', label: 'IP commune' },
        { ip: '192.168.0.100', label: 'Réseau alternatif' },
        { ip: '10.0.0.100', label: 'Réseau local' },
        { ip: '127.0.0.1', label: 'Local (test)' }
    ];

    // Filter out current IP
    return suggestions.filter(s => s.ip !== formData.value.ipAddress);
});

// Methods
const isValidIP = (ip: string): boolean => {
    const ipRegex = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/;
    const match = ip.match(ipRegex);
    
    if (!match) return false;
    
    return match.slice(1).every(num => {
        const n = parseInt(num, 10);
        return n >= 0 && n <= 255;
    });
};

const validateIP = () => {
    const ip = formData.value.ipAddress.trim();
    
    if (!ip) {
        errors.value.ipAddress = 'L\'adresse IP est requise';
        return;
    }
    
    if (!isValidIP(ip)) {
        errors.value.ipAddress = 'Format d\'IP invalide (ex: 192.168.1.100)';
        return;
    }
    
    delete errors.value.ipAddress;
};

const validateForm = () => {
    errors.value = {};
    
    // Validate name
    if (!formData.value.name.trim()) {
        errors.value.name = 'Le nom est requis';
    }
    
    // Validate IP
    validateIP();
    
    // Validate port
    if (formData.value.port < 1 || formData.value.port > 65535) {
        errors.value.port = 'Le port doit être entre 1 et 65535';
    }
};

const applySuggestion = (suggestion: IPSuggestion) => {
    formData.value.ipAddress = suggestion.ip;
    validateIP();
    showSuggestions.value = false;
};

const testConnection = async () => {
    testing.value = true;
    testResult.value = null;
    
    try {
        emit('test', { ...formData.value });
        
        // Simulate test result (replace with actual response)
        await new Promise(resolve => setTimeout(resolve, 1500));
        
        testResult.value = {
            type: 'success',
            message: '✅ Connexion réussie!'
        };
    } catch (error) {
        testResult.value = {
            type: 'error',
            message: '❌ Connexion échouée: ' + error
        };
    } finally {
        testing.value = false;
    }
};

const handleSubmit = async () => {
    validateForm();
    
    if (!isValidForm.value) return;
    
    submitting.value = true;
    
    try {
        await nextTick();
        emit('save', { ...formData.value });
        handleClose();
    } catch (error) {
        console.error('Error saving device:', error);
    } finally {
        submitting.value = false;
    }
};

const handleClose = () => {
    emit('close');
    resetForm();
};

const handleOverlayClick = (event: MouseEvent) => {
    if (event.target === event.currentTarget) {
        handleClose();
    }
};

const resetForm = () => {
    formData.value = {
        name: '',
        ipAddress: '',
        port: 8081
    };
    errors.value = {};
    testResult.value = null;
    showSuggestions.value = true;
};

// Watch for device prop changes
watch(() => props.device, (device) => {
    if (device) {
        formData.value = { ...device };
        showSuggestions.value = false;
    } else {
        resetForm();
    }
}, { immediate: true, deep: true });

// Watch for IP changes to show/hide suggestions
watch(() => formData.value.ipAddress, () => {
    if (formData.value.ipAddress.length > 0) {
        showSuggestions.value = false;
    }
});
</script>

<style scoped>
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.75);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
    backdrop-filter: blur(4px);
}

.modal-container {
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 12px;
    width: 100%;
    max-width: 500px;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
}

.modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 1px solid #30363d;
}

.modal-title {
    font-size: 1.25rem;
    font-weight: 600;
    color: #f0f6fc;
    margin: 0;
}

.close-btn {
    background: none;
    border: none;
    font-size: 1.25rem;
    cursor: pointer;
    color: #8b949e;
    padding: 0.25rem;
    border-radius: 4px;
    transition: all 0.2s ease;
}

.close-btn:hover {
    color: #f85149;
    background: #30363d;
}

.modal-form {
    padding: 1.5rem;
}

.form-group {
    margin-bottom: 1.5rem;
}

.form-label {
    display: block;
    font-weight: 500;
    color: #f0f6fc;
    margin-bottom: 0.5rem;
    font-size: 0.875rem;
}

.form-input {
    width: 100%;
    padding: 0.75rem;
    background: #0d1117;
    border: 1px solid #30363d;
    border-radius: 6px;
    color: #f0f6fc;
    font-size: 0.875rem;
    transition: all 0.2s ease;
}

.form-input:focus {
    outline: none;
    border-color: #58a6ff;
    box-shadow: 0 0 0 2px rgba(88, 166, 255, 0.2);
}

.form-input.error {
    border-color: #f85149;
}

.form-input.error:focus {
    box-shadow: 0 0 0 2px rgba(248, 81, 73, 0.2);
}

.form-hint {
    font-size: 0.75rem;
    color: #8b949e;
    margin-top: 0.25rem;
}

.error-message {
    color: #f85149;
    font-size: 0.75rem;
    margin-top: 0.25rem;
}

.suggestions-section {
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: #0d1117;
    border: 1px solid #30363d;
    border-radius: 8px;
}

.suggestions-title {
    font-size: 0.875rem;
    color: #f0f6fc;
    margin: 0 0 0.75rem 0;
    font-weight: 500;
}

.suggestions-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 0.5rem;
}

.suggestion-btn {
    background: #21262d;
    border: 1px solid #30363d;
    border-radius: 6px;
    padding: 0.5rem;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
}

.suggestion-btn:hover {
    background: #30363d;
    border-color: #58a6ff;
}

.suggestion-ip {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
    font-size: 0.75rem;
    color: #f0f6fc;
    font-weight: 500;
}

.suggestion-label {
    font-size: 0.6rem;
    color: #8b949e;
    margin-top: 0.125rem;
}

.test-section {
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: #0d1117;
    border: 1px solid #30363d;
    border-radius: 8px;
}

.test-btn {
    width: 100%;
    margin-bottom: 0.75rem;
}

.test-result {
    padding: 0.75rem;
    border-radius: 6px;
    font-size: 0.875rem;
    text-align: center;
}

.test-result.success {
    background: rgba(35, 134, 54, 0.2);
    border: 1px solid #238636;
    color: #7de88e;
}

.test-result.error {
    background: rgba(248, 81, 73, 0.2);
    border: 1px solid #f85149;
    color: #ff9494;
}

.test-result.warning {
    background: rgba(187, 128, 9, 0.2);
    border: 1px solid #bb8009;
    color: #f2cc60;
}

.form-actions {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
    padding-top: 1rem;
    border-top: 1px solid #30363d;
}

.btn {
    padding: 0.75rem 1.5rem;
    border: 1px solid #30363d;
    background: #21262d;
    color: #f0f6fc;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: all 0.2s ease;
    text-align: center;
    text-decoration: none;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
}

.btn:hover:not(:disabled) {
    background: #30363d;
    border-color: #58a6ff;
}

.btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.btn-primary {
    background: #238636;
    border-color: #238636;
}

.btn-primary:hover:not(:disabled) {
    background: #2ea043;
}

.btn-secondary {
    background: #6e7681;
    border-color: #6e7681;
}

.btn-secondary:hover:not(:disabled) {
    background: #8b949e;
}

.btn-info {
    background: #7c3aed;
    border-color: #7c3aed;
}

.btn-info:hover:not(:disabled) {
    background: #8b5cf6;
}

/* Responsive */
@media (max-width: 600px) {
    .modal-overlay {
        padding: 0.5rem;
    }
    
    .modal-header,
    .modal-form {
        padding: 1rem;
    }
    
    .suggestions-grid {
        grid-template-columns: 1fr;
    }
    
    .form-actions {
        flex-direction: column;
    }
}
</style> 