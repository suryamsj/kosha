const STORAGE_KEY = 'kosha-theme';

type Theme = 'light' | 'dark';

function getInitialTheme(): Theme {
	const stored = localStorage.getItem(STORAGE_KEY);
	if (stored === 'light' || stored === 'dark') return stored;
	return matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function createThemeStore() {
	let theme = $state<Theme>('light');

	function apply() {
		document.documentElement.dataset.theme = theme;
		localStorage.setItem(STORAGE_KEY, theme);
	}

	function init() {
		theme = getInitialTheme();
		apply();
	}

	function toggle() {
		theme = theme === 'light' ? 'dark' : 'light';
		apply();
	}

	return {
		get theme() {
			return theme;
		},
		init,
		toggle,
	};
}

export const themeStore = createThemeStore();
