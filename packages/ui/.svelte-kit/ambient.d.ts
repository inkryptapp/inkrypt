
// this file is generated — do not edit it


/// <reference types="@sveltejs/kit" />

/**
 * Environment variables [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env`. Like [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), this module cannot be imported into client-side code. This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured).
 * 
 * _Unlike_ [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), the values exported from this module are statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * ```ts
 * import { API_KEY } from '$env/static/private';
 * ```
 * 
 * Note that all environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * 
 * ```
 * MY_FEATURE_FLAG=""
 * ```
 * 
 * You can override `.env` values from the command line like so:
 * 
 * ```sh
 * MY_FEATURE_FLAG="enabled" npm run dev
 * ```
 */
declare module '$env/static/private' {
	export const NVM_INC: string;
	export const npm_package_devDependencies__tailwindcss_typography: string;
	export const SSH_AGENT_PID: string;
	export const TERM_PROGRAM: string;
	export const NODE: string;
	export const NVM_CD_FLAGS: string;
	export const INIT_CWD: string;
	export const npm_package_devDependencies_typescript: string;
	export const SHELL: string;
	export const TERM: string;
	export const npm_package_devDependencies_vite: string;
	export const npm_package_devDependencies__lucide_svelte: string;
	export const TMPDIR: string;
	export const HOMEBREW_REPOSITORY: string;
	export const npm_package_devDependencies_clsx: string;
	export const TERM_PROGRAM_VERSION: string;
	export const ZDOTDIR: string;
	export const ORIGINAL_XDG_CURRENT_DESKTOP: string;
	export const MallocNanoZone: string;
	export const TERM_SESSION_ID: string;
	export const npm_package_private: string;
	export const npm_package_exports___svelte: string;
	export const npm_package_devDependencies__sveltejs_kit: string;
	export const npm_config_registry: string;
	export const PNPM_HOME: string;
	export const ZSH: string;
	export const USER: string;
	export const NVM_DIR: string;
	export const npm_package_description: string;
	export const npm_package_devDependencies_tailwind_variants: string;
	export const LS_COLORS: string;
	export const npm_package_license: string;
	export const npm_package_scripts_check_watch: string;
	export const COMMAND_MODE: string;
	export const npm_package_exports___transitions__: string;
	export const PNPM_SCRIPT_SRC_DIR: string;
	export const npm_package_devDependencies__sveltejs_adapter_static: string;
	export const npm_package_devDependencies__tailwindcss_vite: string;
	export const SSH_AUTH_SOCK: string;
	export const VSCODE_PROFILE_INITIALIZED: string;
	export const __CF_USER_TEXT_ENCODING: string;
	export const TERM_FEATURES: string;
	export const npm_execpath: string;
	export const PAGER: string;
	export const npm_package_devDependencies_svelte: string;
	export const LSCOLORS: string;
	export const npm_package_scripts_sync: string;
	export const npm_config_frozen_lockfile: string;
	export const TERMINFO_DIRS: string;
	export const PATH: string;
	export const LaunchInstanceID: string;
	export const npm_config_engine_strict: string;
	export const USER_ZDOTDIR: string;
	export const __CFBundleIdentifier: string;
	export const npm_package_exports___utils_js: string;
	export const npm_config_auto_install_peers: string;
	export const PWD: string;
	export const npm_package_devDependencies_tailwindcss: string;
	export const npm_command: string;
	export const npm_lifecycle_event: string;
	export const LANG: string;
	export const npm_package_name: string;
	export const npm_package_scripts_ui: string;
	export const npm_package_devDependencies__sveltejs_vite_plugin_svelte: string;
	export const ITERM_PROFILE: string;
	export const NODE_PATH: string;
	export const npm_package_exports___types: string;
	export const npm_package_exports___actions__: string;
	export const BUN_INSPECT_CONNECT_TO: string;
	export const VSCODE_GIT_ASKPASS_EXTRA_ARGS: string;
	export const XPC_FLAGS: string;
	export const npm_package_exports___styles__: string;
	export const npm_package_devDependencies_tw_animate_css: string;
	export const npm_package_devDependencies_tailwind_merge: string;
	export const npm_config_node_gyp: string;
	export const XPC_SERVICE_NAME: string;
	export const npm_package_version: string;
	export const npm_package_exports___hooks__: string;
	export const VSCODE_INJECTION: string;
	export const npm_package_devDependencies_svelte_check: string;
	export const COLORFGBG: string;
	export const HOME: string;
	export const SHLVL: string;
	export const PYENV_SHELL: string;
	export const npm_package_type: string;
	export const npm_package_exports___attachments__: string;
	export const VSCODE_GIT_ASKPASS_MAIN: string;
	export const LC_TERMINAL_VERSION: string;
	export const HOMEBREW_PREFIX: string;
	export const ITERM_SESSION_ID: string;
	export const npm_package_exports___default: string;
	export const LESS: string;
	export const LOGNAME: string;
	export const npm_lifecycle_script: string;
	export const VSCODE_GIT_IPC_HANDLE: string;
	export const npm_package_exports___components__: string;
	export const NVM_BIN: string;
	export const BUN_INSTALL: string;
	export const npm_config_user_agent: string;
	export const VSCODE_GIT_ASKPASS_NODE: string;
	export const GIT_ASKPASS: string;
	export const HOMEBREW_CELLAR: string;
	export const INFOPATH: string;
	export const LC_TERMINAL: string;
	export const npm_package_scripts_prepare: string;
	export const NPM_READ_TOKEN: string;
	export const SECURITYSESSIONID: string;
	export const npm_package_scripts_check: string;
	export const COLORTERM: string;
	export const npm_node_execpath: string;
}

/**
 * Similar to [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private), except that it only includes environment variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`), and can therefore safely be exposed to client-side code.
 * 
 * Values are replaced statically at build time.
 * 
 * ```ts
 * import { PUBLIC_BASE_URL } from '$env/static/public';
 * ```
 */
declare module '$env/static/public' {
	
}

/**
 * This module provides access to runtime environment variables, as defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`. This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured).
 * 
 * This module cannot be imported into client-side code.
 * 
 * Dynamic environment variables cannot be used during prerendering.
 * 
 * ```ts
 * import { env } from '$env/dynamic/private';
 * console.log(env.DEPLOYMENT_SPECIFIC_VARIABLE);
 * ```
 * 
 * > [!NOTE] In `dev`, `$env/dynamic` always includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 */
declare module '$env/dynamic/private' {
	export const env: {
		NVM_INC: string;
		npm_package_devDependencies__tailwindcss_typography: string;
		SSH_AGENT_PID: string;
		TERM_PROGRAM: string;
		NODE: string;
		NVM_CD_FLAGS: string;
		INIT_CWD: string;
		npm_package_devDependencies_typescript: string;
		SHELL: string;
		TERM: string;
		npm_package_devDependencies_vite: string;
		npm_package_devDependencies__lucide_svelte: string;
		TMPDIR: string;
		HOMEBREW_REPOSITORY: string;
		npm_package_devDependencies_clsx: string;
		TERM_PROGRAM_VERSION: string;
		ZDOTDIR: string;
		ORIGINAL_XDG_CURRENT_DESKTOP: string;
		MallocNanoZone: string;
		TERM_SESSION_ID: string;
		npm_package_private: string;
		npm_package_exports___svelte: string;
		npm_package_devDependencies__sveltejs_kit: string;
		npm_config_registry: string;
		PNPM_HOME: string;
		ZSH: string;
		USER: string;
		NVM_DIR: string;
		npm_package_description: string;
		npm_package_devDependencies_tailwind_variants: string;
		LS_COLORS: string;
		npm_package_license: string;
		npm_package_scripts_check_watch: string;
		COMMAND_MODE: string;
		npm_package_exports___transitions__: string;
		PNPM_SCRIPT_SRC_DIR: string;
		npm_package_devDependencies__sveltejs_adapter_static: string;
		npm_package_devDependencies__tailwindcss_vite: string;
		SSH_AUTH_SOCK: string;
		VSCODE_PROFILE_INITIALIZED: string;
		__CF_USER_TEXT_ENCODING: string;
		TERM_FEATURES: string;
		npm_execpath: string;
		PAGER: string;
		npm_package_devDependencies_svelte: string;
		LSCOLORS: string;
		npm_package_scripts_sync: string;
		npm_config_frozen_lockfile: string;
		TERMINFO_DIRS: string;
		PATH: string;
		LaunchInstanceID: string;
		npm_config_engine_strict: string;
		USER_ZDOTDIR: string;
		__CFBundleIdentifier: string;
		npm_package_exports___utils_js: string;
		npm_config_auto_install_peers: string;
		PWD: string;
		npm_package_devDependencies_tailwindcss: string;
		npm_command: string;
		npm_lifecycle_event: string;
		LANG: string;
		npm_package_name: string;
		npm_package_scripts_ui: string;
		npm_package_devDependencies__sveltejs_vite_plugin_svelte: string;
		ITERM_PROFILE: string;
		NODE_PATH: string;
		npm_package_exports___types: string;
		npm_package_exports___actions__: string;
		BUN_INSPECT_CONNECT_TO: string;
		VSCODE_GIT_ASKPASS_EXTRA_ARGS: string;
		XPC_FLAGS: string;
		npm_package_exports___styles__: string;
		npm_package_devDependencies_tw_animate_css: string;
		npm_package_devDependencies_tailwind_merge: string;
		npm_config_node_gyp: string;
		XPC_SERVICE_NAME: string;
		npm_package_version: string;
		npm_package_exports___hooks__: string;
		VSCODE_INJECTION: string;
		npm_package_devDependencies_svelte_check: string;
		COLORFGBG: string;
		HOME: string;
		SHLVL: string;
		PYENV_SHELL: string;
		npm_package_type: string;
		npm_package_exports___attachments__: string;
		VSCODE_GIT_ASKPASS_MAIN: string;
		LC_TERMINAL_VERSION: string;
		HOMEBREW_PREFIX: string;
		ITERM_SESSION_ID: string;
		npm_package_exports___default: string;
		LESS: string;
		LOGNAME: string;
		npm_lifecycle_script: string;
		VSCODE_GIT_IPC_HANDLE: string;
		npm_package_exports___components__: string;
		NVM_BIN: string;
		BUN_INSTALL: string;
		npm_config_user_agent: string;
		VSCODE_GIT_ASKPASS_NODE: string;
		GIT_ASKPASS: string;
		HOMEBREW_CELLAR: string;
		INFOPATH: string;
		LC_TERMINAL: string;
		npm_package_scripts_prepare: string;
		NPM_READ_TOKEN: string;
		SECURITYSESSIONID: string;
		npm_package_scripts_check: string;
		COLORTERM: string;
		npm_node_execpath: string;
		[key: `PUBLIC_${string}`]: undefined;
		[key: `${string}`]: string | undefined;
	}
}

/**
 * Similar to [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), but only includes variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`), and can therefore safely be exposed to client-side code.
 * 
 * Note that public dynamic environment variables must all be sent from the server to the client, causing larger network requests — when possible, use `$env/static/public` instead.
 * 
 * Dynamic environment variables cannot be used during prerendering.
 * 
 * ```ts
 * import { env } from '$env/dynamic/public';
 * console.log(env.PUBLIC_DEPLOYMENT_SPECIFIC_VARIABLE);
 * ```
 */
declare module '$env/dynamic/public' {
	export const env: {
		[key: `PUBLIC_${string}`]: string | undefined;
	}
}
