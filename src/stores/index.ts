import { useAudioStore } from './audio';
import { useColorsStore } from './colors';
import { useEffectsStore } from './effects';
import { useFramesStore } from './frames';
import { useLEDStore } from './led';
import { useLogsStore } from './logs';
import { usePresetsStore } from './presets';
import { useSystemStore } from './system';

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

// Utility function to reset all stores
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

			stores.forEach((store) => store.reset());
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
