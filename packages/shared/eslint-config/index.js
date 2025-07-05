module.exports = {
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:unicorn/all",
    "plugin:vue/vue3-recommended",
    "prettier"
  ],
  ignorePatterns: ["dist"],
  parser: "vue-eslint-parser",
  parserOptions: {
    parser: "@typescript-eslint/parser",
    sourceType: "module",
    ecmaVersion: 2020
  },
  plugins: [
    "@typescript-eslint",
    "prettier",
    "vue",
    "unused-imports"
  ],
  rules: {
    "prettier/prettier": "error",
    "unicorn/filename-case": [
      "error",
      {
        cases: {
          camelCase: true,
          pascalCase: true,
          kebabCase: true
        }
      }
    ],
    "unicorn/no-keyword-prefix": "off",
    "unicorn/prevent-abbreviations": "off",
    "no-unused-vars": ["warn", { argsIgnorePattern: "^_" }],
    "unused-imports/no-unused-imports": "error",
    "unicorn/prefer-module": "off"
  }
}
