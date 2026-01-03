declare global {
  interface Window {
    __TAURI__?: any;
  }
}

declare module '@tauri-apps/api/core' {
  export function invoke<T = any>(cmd: string, args?: any): Promise<T>;
}

declare module '@tauri-apps/api/event' {
  export function listen<T = any>(event: string, handler: (event: { payload: T }) => void): Promise<{ (): void }>;
}

declare module '@tauri-apps/plugin-dialog' {
  export function open(options?: any): Promise<string | null>;
  export function save(options?: any): Promise<string | null>;
}

declare module '@zerodevx/svelte-toast' {
  export const toast: {
    push: (message: string, options?: any) => void;
  };
  export const SvelteToast: any;
}