/// <reference types="vite/client" />
/// <reference types="@tauri-apps/api/tauri" />

declare module '*.md' {
  const content: string;
  export default content;
}
