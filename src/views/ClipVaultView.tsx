import { useState, useEffect, useCallback, useRef } from 'react';
import { useClips } from '../hooks/useClips';
import { setTheme } from '../lib/theme';
import {
  type ContentType,
  snipScreen,
  minimizeWindow,
  toggleMaximize,
  quitApp,
  startDrag,
  viewImageClip,
  revealPath,
} from '../api/clips';
import {
  Search,
  Sun,
  Moon,
  Star,
  Trash2,
  FileText,
  Image,
  FolderOpen,
  ClipboardCopy,
  Copy,
  X,
  Loader2,
  Scissors,
  Minus,
  Square,
  Eye,
} from 'lucide-react';

function dataUriToBlob(dataUri: string): string {
  try {
    const b64 = dataUri.split(',')[1];
    const binary = atob(b64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    const blob = new Blob([bytes], { type: 'image/png' });
    return URL.createObjectURL(blob);
  } catch {
    return dataUri;
  }
}

const TYPE_CONFIG: Record<
  ContentType,
  { label: string; icon: React.ReactNode; color: string; dot: string; bg: string }
> = {
  text: {
    label: '文本',
    icon: <FileText className="h-3.5 w-3.5" />,
    color: 'text-blue-500',
    dot: 'bg-blue-500',
    bg: 'bg-blue-50 dark:bg-blue-900/20',
  },
  image: {
    label: '图片',
    icon: <Image className="h-3.5 w-3.5" />,
    color: 'text-amber-500',
    dot: 'bg-amber-500',
    bg: 'bg-amber-50 dark:bg-amber-900/20',
  },
  file_path: {
    label: '路径',
    icon: <FolderOpen className="h-3.5 w-3.5" />,
    color: 'text-emerald-500',
    dot: 'bg-emerald-500',
    bg: 'bg-emerald-50 dark:bg-emerald-900/20',
  },
};

const FILTER_TABS: { value: ContentType | undefined; label: string }[] = [
  { value: undefined, label: '全部' },
  { value: 'text', label: '文本' },
  { value: 'image', label: '图片' },
  { value: 'file_path', label: '路径' },
];

function formatTime(ts: number): string {
  const diffMs = Date.now() - ts;
  const min = Math.floor(diffMs / 60000);
  if (min < 1) return '刚刚';
  if (min < 60) return `${min}分钟前`;
  const hr = Math.floor(min / 60);
  if (hr < 24) return `${hr}小时前`;
  const d = Math.floor(hr / 24);
  if (d < 7) return `${d}天前`;
  return new Date(ts).toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' });
}

function HighlightText({ text, query }: { text: string; query: string }) {
  if (!query.trim()) return <>{text}</>;
  const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const parts = text.split(new RegExp(`(${escaped})`, 'gi'));
  return (
    <>
      {parts.map((part, i) =>
        part.toLowerCase() === query.toLowerCase() ? (
          <mark key={i} className="bg-yellow-200/80 dark:bg-yellow-500/30 rounded-sm px-[1px]">
            {part}
          </mark>
        ) : (
          part
        ),
      )}
    </>
  );
}

function TrafficLight({ variant, onClick }: { variant: 'close' | 'minimize' | 'maximize'; onClick: () => void }) {
  const colors = {
    close: 'bg-[#ff5f57] hover:bg-[#ff3b30]',
    minimize: 'bg-[#febc2e] hover:bg-[#f5a623]',
    maximize: 'bg-[#28c840] hover:bg-[#1db954]',
  };
  const icons: Record<string, React.ReactNode> = {
    close: <X className="h-[7px] w-[7px] opacity-0 group-hover:opacity-100" />,
    minimize: <Minus className="h-[7px] w-[7px] opacity-0 group-hover:opacity-100" />,
    maximize: <Square className="h-[7px] w-[7px] opacity-0 group-hover:opacity-100" />,
  };
  return (
    <button
      onClick={onClick}
      className={`group flex items-center justify-center h-[12px] w-[12px] rounded-full ${colors[variant]} transition-all duration-150`}
    >
      <span className="text-black/80">{icons[variant]}</span>
    </button>
  );
}

export function ClipVaultView() {
  const { clips, loading, error, search, toggleFav, remove, paste, filterType, setFilterType, loadClipsSilent } =
    useClips();
  const [query, setQuery] = useState('');
  const [darkMode, setDarkMode] = useState(() =>
    document.documentElement.classList.contains('dark'),
  );
  const [focusIdx, setFocusIdx] = useState(-1);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [snipping, setSnipping] = useState(false);
  const [showQuitConfirm, setShowQuitConfirm] = useState(false);
  const listRef = useRef<HTMLUListElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const timer = setTimeout(() => search(query), 300);
    return () => clearTimeout(timer);
  }, [query, search]);

  useEffect(() => {
    loadClipsSilent();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [filterType]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleToggleDark = useCallback(async () => {
    const next = !darkMode;
    setDarkMode(next);
    await setTheme(next ? 'dark' : 'light');
  }, [darkMode]);

  const handleCopy = useCallback(
    async (id: string) => {
      const clip = clips.find((c) => c.id === id);
      if (!clip || clip.content_type !== 'text') return;
      await paste(id);
      setCopiedId(id);
      setTimeout(() => setCopiedId(null), 1500);
    },
    [clips, paste],
  );

  const handleSnip = useCallback(async () => {
    try {
      setSnipping(true);
      await snipScreen();
    } catch {
      // cancelled or failed
    } finally {
      setSnipping(false);
    }
  }, []);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setFocusIdx((i) => Math.min(i + 1, clips.length - 1));
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setFocusIdx((i) => Math.max(i - 1, -1));
      } else if (e.key === 'Enter' && focusIdx >= 0 && focusIdx < clips.length) {
        e.preventDefault();
        const clip = clips[focusIdx];
        if (clip.content_type === 'text') handleCopy(clip.id);
      } else if (e.key === 'Escape') {
        setQuery('');
        setFocusIdx(-1);
      }
    },
    [clips, focusIdx, handleCopy],
  );

  useEffect(() => {
    if (focusIdx >= 0 && listRef.current) {
      const items = listRef.current.querySelectorAll('[data-clip-item]');
      items[focusIdx]?.scrollIntoView({ block: 'nearest' });
    }
  }, [focusIdx]);

  const titleBarRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = titleBarRef.current;
    if (!el) return;

    const onMouseDown = (e: MouseEvent) => {
      if ((e.target as HTMLElement).closest('button')) return;
      startDrag().catch(() => {});
    };

    el.addEventListener('mousedown', onMouseDown);
    return () => el.removeEventListener('mousedown', onMouseDown);
  }, []);

  return (
    <div className="flex h-screen flex-col select-none rounded-xl overflow-hidden" onKeyDown={handleKeyDown}>
      {/* Title bar */}
      <div
        ref={titleBarRef}
        className="group h-10 shrink-0 flex items-center gap-1.5 px-3 bg-[#f8f8fa] dark:bg-[#1c1c20] cursor-grab active:cursor-grabbing"
      >
        {/* Traffic lights — no wrapper div, buttons are direct children */}
        <TrafficLight variant="close" onClick={() => setShowQuitConfirm(true)} />
        <TrafficLight variant="minimize" onClick={() => minimizeWindow()} />
        <TrafficLight variant="maximize" onClick={() => toggleMaximize()} />

        {/* Spacer + title */}
        <span className="flex-1 text-center text-[11px] font-medium text-neutral-400 dark:text-neutral-600 select-none">
          ClipVault
        </span>

        {/* Snip button */}
        <button
          onClick={handleSnip}
          disabled={snipping}
          className={`shrink-0 rounded-md p-1.5 transition-all duration-150 ${
            snipping
              ? 'text-blue-500 bg-blue-50 dark:bg-blue-900/20'
              : 'text-neutral-400 hover:text-blue-500 hover:bg-blue-50 dark:hover:bg-blue-900/20'
          }`}
          aria-label="截图"
          title="截图"
        >
          <Scissors className={`h-3.5 w-3.5 ${snipping ? 'animate-pulse' : ''}`} />
        </button>
      </div>

      {/* Search bar */}
      <div className="shrink-0 px-3 pb-2 bg-[#f8f8fa] dark:bg-[#1c1c20]">
        <div className="flex items-center gap-2 rounded-lg bg-white dark:bg-[#2a2a2e] px-3 py-2 ring-1 ring-black/[0.04] dark:ring-white/[0.06] focus-within:ring-2 focus-within:ring-blue-500/40 transition-shadow">
          <Search className="h-3.5 w-3.5 shrink-0 text-neutral-400" />
          <input
            ref={inputRef}
            type="search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="搜索剪贴板历史..."
            aria-label="搜索"
            className="flex-1 bg-transparent text-[13px] leading-tight focus:outline-none placeholder:text-neutral-400 min-w-0 text-neutral-800 dark:text-neutral-200"
          />
          {query && (
            <button
              onClick={() => setQuery('')}
              className="shrink-0 rounded p-0.5 text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300"
            >
              <X className="h-3 w-3" />
            </button>
          )}
        </div>
      </div>

      {/* Filter tabs */}
      <div className="shrink-0 flex items-center gap-0.5 px-3 pb-2 bg-[#f8f8fa] dark:bg-[#1c1c20]">
        {FILTER_TABS.map((tab) => {
          const active = filterType === tab.value;
          const tc = tab.value ? TYPE_CONFIG[tab.value] : null;
          return (
            <button
              key={String(tab.value)}
              onClick={() => setFilterType(tab.value)}
              className={`flex items-center gap-1.5 rounded-md px-2.5 py-1 text-[11px] font-medium transition-all duration-150 ${
                active
                  ? tc
                    ? `${tc.bg} ${tc.color}`
                    : 'bg-neutral-200/80 dark:bg-neutral-700/60 text-neutral-700 dark:text-neutral-200'
                  : 'text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800/60'
              }`}
            >
              {tc && <span className={`inline-block h-1.5 w-1.5 rounded-full ${tc.dot}`} />}
              {tab.label}
            </button>
          );
        })}
        <div className="flex-1" />
        <button
          onClick={handleToggleDark}
          aria-label={darkMode ? '亮色模式' : '暗色模式'}
          className="rounded-md p-1.5 text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800/60 transition-colors"
        >
          {darkMode ? <Sun className="h-3.5 w-3.5" /> : <Moon className="h-3.5 w-3.5" />}
        </button>
      </div>

      {/* Error banner */}
      {error && (
        <div
          role="alert"
          className="mx-3 mb-2 rounded-lg bg-red-50 dark:bg-red-900/15 border border-red-200/60 dark:border-red-800/30 p-2.5 text-[12px] text-red-600 dark:text-red-400"
        >
          {error}
        </div>
      )}

      {/* Loading */}
      {loading && (
        <div className="flex flex-1 items-center justify-center" aria-live="polite">
          <div className="flex flex-col items-center gap-2">
            <Loader2 className="h-4 w-4 animate-spin text-blue-500" />
            <span className="text-[11px] text-neutral-400">加载中...</span>
          </div>
        </div>
      )}

      {/* Empty state */}
      {!loading && clips.length === 0 && (
        <div className="flex flex-1 flex-col items-center justify-center px-8 py-12 bg-[#f8f8fa] dark:bg-[#1c1c20]">
          <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-neutral-100 to-neutral-50 dark:from-neutral-800 dark:to-neutral-800/50 flex items-center justify-center mb-4">
            <ClipboardCopy className="h-7 w-7 text-neutral-300 dark:text-neutral-600" />
          </div>
          <p className="text-neutral-500 dark:text-neutral-400 text-[13px] font-medium mb-1">
            暂无剪贴板记录
          </p>
          <p className="text-neutral-400 dark:text-neutral-600 text-[11px] text-center leading-5">
            复制内容后会自动出现在这里
          </p>
        </div>
      )}

      {/* Clip list */}
      {!loading && clips.length > 0 && (
        <ul
          ref={listRef}
          role="listbox"
          aria-label="剪贴板记录"
          className="flex-1 overflow-y-auto scroll-smooth bg-white dark:bg-[#242428]"
        >
          {clips.map((clip, idx) => {
            const tc = TYPE_CONFIG[clip.content_type];
            const isFocused = idx === focusIdx;
            const isCopied = copiedId === clip.id;
            return (
              <li
                key={clip.id}
                role="option"
                aria-selected={isFocused}
                data-clip-item
                className={`group relative flex items-start gap-2.5 px-3 py-2.5 cursor-default transition-colors duration-100 ${
                  isFocused
                    ? 'bg-blue-50/90 dark:bg-blue-900/15'
                    : 'hover:bg-neutral-50 dark:hover:bg-neutral-800/30'
                } ${idx < clips.length - 1 ? 'border-b border-neutral-100 dark:border-neutral-800/60' : ''}`}
                onClick={() => setFocusIdx(idx)}
                onDoubleClick={() => {
                  if (clip.content_type === 'text') handleCopy(clip.id);
                  else if (clip.content_type === 'image') viewImageClip(clip.id);
                  else if (clip.content_type === 'file_path') revealPath(clip.id);
                }}
              >
                <span className={`mt-1.5 shrink-0 h-2 w-2 rounded-full ${tc.dot}`} title={tc.label} />

                <div className="min-w-0 flex-1">
                  {clip.content_type === 'image' && clip.preview.startsWith('data:image/') ? (
                    <img
                      src={dataUriToBlob(clip.preview)}
                      alt="剪贴板图片"
                      className="max-h-24 rounded-md border border-neutral-100 dark:border-neutral-800 bg-checkerboard"
                      loading="lazy"
                    />
                  ) : (
                  <p className="text-[13px] leading-[1.5] text-neutral-700 dark:text-neutral-300 line-clamp-2">
                    <HighlightText text={clip.preview} query={query} />
                  </p>
                  )}
                  <div className="flex items-center gap-2 mt-1">
                    <span className={`inline-flex items-center gap-1 text-[10px] font-medium ${tc.color}`}>
                      {tc.icon}
                      {tc.label}
                    </span>
                    <span className="text-[10px] text-neutral-400 dark:text-neutral-600">
                      {formatTime(clip.created_at)}
                    </span>
                  </div>
                </div>

                <div className="shrink-0 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity duration-150">
                  {clip.content_type === 'image' && (
                  <button
                    onClick={(e) => { e.stopPropagation(); viewImageClip(clip.id); }}
                    aria-label="查看图片"
                    className="rounded-md p-1 text-neutral-400 hover:text-amber-500 hover:bg-amber-50 dark:hover:bg-amber-900/20 transition-colors"
                  >
                    <Eye className="h-3 w-3" />
                  </button>
                  )}
                  {clip.content_type === 'file_path' && (
                  <button
                    onClick={(e) => { e.stopPropagation(); revealPath(clip.id); }}
                    aria-label="在 Finder 中显示"
                    className="rounded-md p-1 text-neutral-400 hover:text-emerald-500 hover:bg-emerald-50 dark:hover:bg-emerald-900/20 transition-colors"
                  >
                    <FolderOpen className="h-3 w-3" />
                  </button>
                  )}
                  {clip.content_type === 'text' && (
                  <button
                    onClick={(e) => { e.stopPropagation(); handleCopy(clip.id); }}
                    aria-label="粘贴"
                    className={`rounded-md p-1 transition-colors ${
                      isCopied
                        ? 'text-green-500 bg-green-50 dark:bg-green-900/20'
                        : 'text-neutral-400 hover:text-blue-500 hover:bg-blue-50 dark:hover:bg-blue-900/20'
                    }`}
                  >
                    <Copy className="h-3 w-3" />
                  </button>
                  )}
                  <button
                    onClick={(e) => { e.stopPropagation(); toggleFav(clip.id); }}
                    aria-label={clip.is_favorite ? '取消收藏' : '收藏'}
                    className={`rounded-md p-1 transition-colors ${
                      clip.is_favorite
                        ? 'text-amber-400 hover:text-amber-500 bg-amber-50 dark:bg-amber-900/10'
                        : 'text-neutral-300 dark:text-neutral-600 hover:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-900/10'
                    }`}
                  >
                    <Star className={`h-3 w-3 ${clip.is_favorite ? 'fill-current' : ''}`} />
                  </button>
                  <button
                    onClick={(e) => { e.stopPropagation(); remove(clip.id); }}
                    aria-label="删除"
                    className="rounded-md p-1 text-neutral-300 dark:text-neutral-600 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/15 transition-colors"
                  >
                    <Trash2 className="h-3 w-3" />
                  </button>
                </div>

                {isCopied && (
                  <span className="absolute right-3 top-1 rounded bg-green-500 px-1.5 py-0.5 text-[9px] font-medium text-white shadow-sm">
                    已粘贴
                  </span>
                )}
              </li>
            );
          })}
        </ul>
      )}

      {/* Footer */}
      {!loading && clips.length > 0 && (
        <div className="shrink-0 px-3 py-1.5 bg-[#f8f8fa] dark:bg-[#1c1c20] border-t border-neutral-100 dark:border-neutral-800/40 flex items-center justify-between text-[10px] text-neutral-400 dark:text-neutral-600">
          <span>{clips.length} 条记录</span>
          <div className="flex items-center gap-1.5">
            <kbd className="rounded bg-white dark:bg-neutral-800 px-1 py-0.5 text-[9px] font-mono shadow-sm ring-1 ring-black/[0.04] dark:ring-white/[0.06]">
              ↵
            </kbd>
            <span>粘贴</span>
            <span className="text-neutral-300 dark:text-neutral-700">·</span>
            <span>↑↓ 导航</span>
            <span className="text-neutral-300 dark:text-neutral-700">·</span>
            <span>Esc 清除</span>
          </div>
        </div>
      )}

      {showQuitConfirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30 rounded-xl">
          <div className="bg-white dark:bg-[#2a2a2e] rounded-xl shadow-xl p-5 mx-6 max-w-xs w-full">
            <p className="text-[13px] font-medium text-neutral-800 dark:text-neutral-200 mb-4">确认退出 ClipVault？</p>
            <div className="flex gap-2 justify-end">
              <button
                onClick={() => setShowQuitConfirm(false)}
                className="rounded-lg px-3.5 py-1.5 text-[12px] font-medium text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-700/60 transition-colors"
              >
                取消
              </button>
              <button
                onClick={() => quitApp()}
                className="rounded-lg px-3.5 py-1.5 text-[12px] font-medium text-white bg-red-500 hover:bg-red-600 transition-colors"
              >
                退出
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
