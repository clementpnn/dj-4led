import { useAudioStore } from '@/stores/audio';
import { useColorsStore } from '@/stores/colors';
import { useEffectsStore } from '@/stores/effects';
import { useFramesStore } from '@/stores/frames';
import { useLEDStore } from '@/stores/led';
import { useLogsStore } from '@/stores/logs';
import { usePresetsStore } from '@/stores/presets';
import { useSystemStore } from '@/stores/system';

// Export individual stores
export {
	useAudioStore,
	useColorsStore,
	useEffectsStore,
	useFramesStore,
	useLEDStore,
	useLogsStore,
	usePresetsStore,
	useSystemStore,
};

// Utility function to reset all stores using Pinia's $reset
export const useResetAllStores = () => {
	const resetAll = () => {
		try {
			const stores = [
				useAudioStore(),
				useEffectsStore(),
				useColorsStore(),
				useLEDStore(),
				useFramesStore(),
				useSystemStore(),
				usePresetsStore(),
				useLogsStore(),
			];

			stores.forEach((store) => {
				if (typeof store.$reset === 'function') {
					store.$reset();
				} else {
					console.warn('Store does not have $reset method:', store);
				}
			});

			console.log('🔄 All stores reset successfully');
		} catch (error) {
			console.error('❌ Error resetting stores:', error);
		}
	};

	return { resetAll };
};

// Utility function to get all store states
export const useStoreDebug = () => {
	const getAllStates = () => {
		try {
			return {
				audio: useAudioStore().$state,
				effects: useEffectsStore().$state,
				colors: useColorsStore().$state,
				led: useLEDStore().$state,
				frames: useFramesStore().$state,
				system: useSystemStore().$state,
				presets: usePresetsStore().$state,
				logs: useLogsStore().$state,
			};
		} catch (error) {
			console.error('❌ Error getting store states:', error);
			return {};
		}
	};

	const logAllStates = () => {
		try {
			const states = getAllStates();
			console.group('🏪 Store States');
			Object.entries(states).forEach(([name, state]) => {
				console.log(`${name}:`, state);
			});
			console.groupEnd();
		} catch (error) {
			console.error('❌ Error logging store states:', error);
		}
	};

	return { getAllStates, logAllStates };
};
