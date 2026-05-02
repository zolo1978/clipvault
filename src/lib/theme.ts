const THEME_KEY = 'clipvault-theme';

export async function initTheme(): Promise<void> {
  const stored = localStorage.getItem(THEME_KEY);
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  const dark = stored ? stored === 'dark' : prefersDark;
  document.documentElement.classList.toggle('dark', dark);
}

export async function setTheme(theme: 'dark' | 'light'): Promise<void> {
  document.documentElement.classList.toggle('dark', theme === 'dark');
  localStorage.setItem(THEME_KEY, theme);
}
