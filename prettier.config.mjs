/**
 * Unfortunately, Biome has a very limited capability of formatting Svelte components.
 * So we use Prettier with the recommended Svelte plugin for *.svelte files.
 *
 * @type {import('prettier').Config}
 */
export default {
  arrowParens: "avoid",
  endOfLine: "lf",
  printWidth: 100,
  tabWidth: 2,
  trailingComma: "all",
  useTabs: false,
  singleQuote: false,
  plugins: ["prettier-plugin-svelte", "prettier-plugin-tailwindcss"],
  overrides: [
    {
      files: "*.svelte",
      options: {
        parser: "svelte",
      },
    },
  ],
};
