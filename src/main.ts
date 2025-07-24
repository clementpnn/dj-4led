import { createPinia } from 'pinia';
import { createApp } from 'vue';
import App from './App.vue';

// Créer l'instance Pinia AVANT l'application
const pinia = createPinia();

// Créer l'application Vue
const app = createApp(App);

// Installer Pinia AVANT de monter l'application
app.use(pinia);

// Monter l'application
app.mount('#app');
