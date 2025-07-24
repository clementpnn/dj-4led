import tsPlugin from '@typescript-eslint/eslint-plugin';
import tsParser from '@typescript-eslint/parser';
import importPlugin from 'eslint-plugin-import';
import prettierPlugin from 'eslint-plugin-prettier';
import vuePlugin from 'eslint-plugin-vue';
import vueA11yPlugin from 'eslint-plugin-vuejs-accessibility';
import vueParser from 'vue-eslint-parser';

const parserOptionsWithTypes = {
	project: './tsconfig.app.json',
	tsconfigRootDir: process.cwd(),
	ecmaVersion: 'latest',
	sourceType: 'module',
};

export default [
	// 🚫 FICHIERS IGNORÉS
	{
		ignores: ['dist/**/*', 'node_modules/**/*', 'coverage/**/*', '**/*.config.{js,ts}'],
	},

	// 🔧 CONFIGURATION GLOBALE
	{
		files: ['**/*.{js,ts,vue}'],
		languageOptions: {
			ecmaVersion: 'latest',
			sourceType: 'module',
			globals: {
				window: 'readonly',
				document: 'readonly',
				console: 'readonly',
				process: 'readonly',
			},
		},
		plugins: {
			prettier: prettierPlugin,
		},
		rules: {
			// Formatage
			'prettier/prettier': 'error',

			// Code propre
			'no-console': process.env.NODE_ENV === 'production' ? 'warn' : 'off',
			'no-debugger': process.env.NODE_ENV === 'production' ? 'error' : 'off',
			'no-var': 'error',
			'prefer-const': 'error',
			'prefer-arrow-callback': 'warn',
			'object-shorthand': 'warn',
		},
	},

	// 🎭 CONFIGURATION VUE
	{
		files: ['**/*.vue'],
		languageOptions: {
			parser: vueParser,
			parserOptions: {
				...parserOptionsWithTypes,
				parser: tsParser,
				extraFileExtensions: ['.vue'],
			},
		},
		plugins: {
			vue: vuePlugin,
			'@typescript-eslint': tsPlugin,
			'vuejs-accessibility': vueA11yPlugin,
			import: importPlugin,
		},
		rules: {
			// 🎨 STRUCTURE DES COMPOSANTS
			'vue/component-tags-order': ['error', { order: ['template', 'script', 'style'] }],
			'vue/block-order': ['error', { order: ['template', 'script', 'style'] }],
			'vue/define-macros-order': ['error', { order: ['defineProps', 'defineEmits', 'defineSlots'] }],
			'vue/component-api-style': ['error', ['script-setup', 'composition']],

			// 🏷️ CONVENTIONS DE NOMMAGE
			'vue/component-name-in-template-casing': ['error', 'PascalCase'],
			'vue/attribute-hyphenation': ['error', 'always'],
			'vue/prop-name-casing': ['error', 'camelCase'],
			'vue/multi-word-component-names': 'off',

			// 🔍 DÉTECTION D'ERREURS
			'vue/no-unused-components': 'warn',
			'vue/no-unused-vars': 'warn',
			'vue/no-unused-properties': [
				'warn',
				{
					groups: ['props', 'data', 'computed', 'methods', 'setup'],
					deepData: true,
				},
			],
			'vue/no-duplicate-attributes': 'error',
			'vue/no-parsing-error': 'error',

			// 📋 BONNES PRATIQUES
			'vue/require-default-prop': 'off',
			'vue/require-prop-types': 'error',
			'vue/no-v-html': 'off',
			'vue/no-v-text-v-html-on-component': 'warn',
			'vue/no-static-inline-styles': 'warn',
			'vue/no-useless-template-attributes': 'error',
			'vue/prefer-true-attribute-shorthand': 'warn',

			// 🎯 PERFORMANCE
			'vue/no-setup-props-destructure': 'error',
			'vue/no-useless-v-bind': 'error',
			'vue/no-useless-mustaches': 'error',
			'vue/v-for-delimiter-style': ['error', 'in'],

			// 🔄 ORDRE ET ORGANISATION
			'vue/attributes-order': 'warn',
			'vue/order-in-components': 'warn',

			// ♿ ACCESSIBILITÉ BASIQUE
			'vuejs-accessibility/click-events-have-key-events': 'warn',
			'vuejs-accessibility/alt-text': 'error',
			'vuejs-accessibility/anchor-has-content': 'error',
			'vuejs-accessibility/form-control-has-label': 'warn',

			// 🔧 TYPESCRIPT DANS VUE
			'@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
			'@typescript-eslint/consistent-type-imports': 'error',
			'@typescript-eslint/prefer-optional-chain': 'warn',
		},
	},

	// 🔷 CONFIGURATION TYPESCRIPT
	{
		files: ['**/*.ts'],
		languageOptions: {
			parser: tsParser,
			parserOptions: parserOptionsWithTypes,
		},
		plugins: {
			'@typescript-eslint': tsPlugin,
			import: importPlugin,
		},
		rules: {
			// 🎯 TYPESCRIPT ESSENTIEL
			'@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
			'@typescript-eslint/ban-ts-comment': 'warn',
			'@typescript-eslint/consistent-type-imports': 'error',
			'@typescript-eslint/consistent-type-definitions': ['error', 'interface'],
			'@typescript-eslint/no-inferrable-types': 'warn',
			'@typescript-eslint/prefer-optional-chain': 'warn',
			'@typescript-eslint/prefer-ts-expect-error': 'error',

			// 🎨 STYLE MODERNE
			'@typescript-eslint/prefer-includes': 'warn',
			'@typescript-eslint/prefer-string-starts-ends-with': 'warn',

			// 📦 GESTION DES IMPORTS
			'import/order': [
				'warn',
				{
					groups: ['builtin', 'external', 'internal', ['parent', 'sibling'], 'index'],
					pathGroups: [
						{ pattern: '@/**', group: 'internal', position: 'after' },
						{ pattern: '@components/**', group: 'internal', position: 'after' },
						{ pattern: '@views/**', group: 'internal', position: 'after' },
						{ pattern: '@stores/**', group: 'internal', position: 'after' },
						{ pattern: '@utils/**', group: 'internal', position: 'after' },
						{ pattern: '@types/**', group: 'internal', position: 'after' },
						{ pattern: './*.{css,scss,sass,less}', group: 'index', position: 'after' },
					],
					pathGroupsExcludedImportTypes: ['builtin'],
					'newlines-between': 'always',
					alphabetize: { order: 'asc', caseInsensitive: true },
				},
			],
			'import/extensions': [
				'error',
				'ignorePackages',
				{ js: 'never', ts: 'never', vue: 'always', scss: 'always' },
			],
			'import/no-unresolved': 'error',
			'import/no-relative-packages': 'error',
			'import/first': 'error',
			'import/no-duplicates': 'error',
		},
		settings: {
			'import/resolver': {
				typescript: {
					alwaysTryTypes: true,
					project: './tsconfig.app.json',
				},
				alias: {
					map: [
						['@', './src'],
						['@components', './src/components'],
						['@views', './src/views'],
						['@stores', './src/stores'],
						['@utils', './src/utils'],
						['@types', './src/types'],
					],
					extensions: ['.ts', '.js', '.vue', '.json'],
				},
			},
		},
	},
];
