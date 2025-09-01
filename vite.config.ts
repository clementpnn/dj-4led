import vue from '@vitejs/plugin-vue';
import path from 'path';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

// Configuration Vite adaptée Tauri, Vue 3 et alias propres
export default defineConfig({
	plugins: [vue()],
	resolve: {
		alias: {
			'@': path.resolve(__dirname, './src'),
			'@components': path.resolve(__dirname, './src/components'),
			'@views': path.resolve(__dirname, './src/views'),
			'@stores': path.resolve(__dirname, './src/stores'),
			'@utils': path.resolve(__dirname, './src/utils'),
			'@types': path.resolve(__dirname, './src/types'),
		},
		extensions: ['.ts', '.js', '.vue', '.json'],
	},
	clearScreen: false,
	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host
			? {
					protocol: 'ws',
					host,
					port: 1421,
				}
			: undefined,
		watch: {
			ignored: ['**/src-tauri/**'],
		},
	},
	build: {
		outDir: 'src-tauri/dist',
		emptyOutDir: true,
		sourcemap: process.env.TAURI_DEBUG ? true : false,
		target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
	},
});
