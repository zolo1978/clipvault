import { useEffect, useState, type ReactNode } from 'react';
import { initTheme } from './lib/theme';
import { ClipVaultView } from './views/ClipVaultView';

// 全局快捷键（Cmd+Shift+V 切换窗口显隐）需要在 Rust 端实现：
// app.plugin(tauri_plugin_global_shortcut::Builder::new().build())?;
// app.global_shortcut().on_shortcut("CmdOrCtrl+Shift+V", |app, _| {
//     if let Some(w) = app.get_webview_window("main") {
//         if w.is_visible().unwrap_or(false) { let _ = w.hide(); }
//         else { let _ = w.show(); let _ = w.set_focus(); }
//     }
// });

function ErrorBoundary({ children }: { children: ReactNode }) {
  const [err, setErr] = useState<string | null>(null);
  useEffect(() => {
    const handler = (e: ErrorEvent) => { setErr(e.error?.message ?? '未知错误'); };
    window.addEventListener('error', handler);
    return () => window.removeEventListener('error', handler);
  }, []);

  if (err) {
    return (
      <div role="alert" className="flex h-screen items-center justify-center bg-red-50 dark:bg-red-950">
        <div className="max-w-md rounded-lg bg-white dark:bg-gray-900 p-8 shadow-lg">
          <h1 className="mb-2 text-xl font-bold text-red-600">应用出错</h1>
          <p className="mb-4 text-sm text-gray-600 dark:text-gray-400">{err}</p>
          <button onClick={() => { setErr(null); window.location.reload(); }} className="rounded bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700">重新加载</button>
        </div>
      </div>
    );
  }
  return <>{children}</>;
}

export default function App() {
  useEffect(() => { initTheme().catch(() => {}); }, []);
  return <ErrorBoundary><ClipVaultView /></ErrorBoundary>;
}
