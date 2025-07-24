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

// Utility function to reset all stores - SAFE VERSION
export const useResetAllStores = () => {
	const resetAll = () => {
		// Utiliser les stores de manière sécurisée
		try {
			const audioStore = useAudioStore();
			const effectsStore = useEffectsStore();
			const colorsStore = useColorsStore();
			const ledStore = useLEDStore();
			const framesStore = useFramesStore();
			const systemStore = useSystemStore();
			const presetsStore = usePresetsStore();
			const logsStore = useLogsStore();

			audioStore.reset();
			effectsStore.reset();
			colorsStore.reset();
			ledStore.reset();
			framesStore.reset();
			systemStore.reset();
			presetsStore.reset();
			logsStore.reset();

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
			const audioStore = useAudioStore();
			const effectsStore = useEffectsStore();
			const colorsStore = useColorsStore();
			const ledStore = useLEDStore();
			const framesStore = useFramesStore();
			const systemStore = useSystemStore();
			const presetsStore = usePresetsStore();
			const logsStore = useLogsStore();

			return {
				audio: audioStore.$state,
				effects: effectsStore.$state,
				colors: colorsStore.$state,
				led: ledStore.$state,
				frames: framesStore.$state,
				system: systemStore.$state,
				presets: presetsStore.$state,
				logs: logsStore.$state,
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
