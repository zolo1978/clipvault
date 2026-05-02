import { useState, useEffect, useCallback } from 'react';
import {
  listClips,
  searchClips,
  toggleFavorite,
  deleteClip,
  pasteClip,
  type ClipSummary,
  type ContentType,
} from '../api/clips';

export function useClips(pageSize = 50) {
  const [clips, setClips] = useState<ClipSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filterType, setFilterType] = useState<ContentType | undefined>();

  const loadClips = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setClips(
        await listClips({
          limit: pageSize,
          offset: 0,
          contentType: filterType,
        }),
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : '加载失败');
    } finally {
      setLoading(false);
    }
  }, [pageSize, filterType]);

  const loadClipsSilent = useCallback(async () => {
    setError(null);
    try {
      setClips(
        await listClips({
          limit: pageSize,
          offset: 0,
          contentType: filterType,
        }),
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : '加载失败');
    }
  }, [pageSize, filterType]);

  const search = useCallback(
    async (query: string) => {
      if (!query.trim()) {
        await loadClips();
        return;
      }
      setLoading(true);
      setError(null);
      try {
        setClips(
          await searchClips({
            query,
            contentType: filterType,
            limit: pageSize,
          }),
        );
      } catch (e) {
        setError(e instanceof Error ? e.message : '搜索失败');
      } finally {
        setLoading(false);
      }
    },
    [pageSize, loadClips, filterType],
  );

  const toggleFav = useCallback(async (id: string) => {
    try {
      const updated = await toggleFavorite(id);
      setClips((prev) =>
        prev.map((c) => (c.id === id ? { ...c, is_favorite: updated.is_favorite } : c)),
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : '切换收藏失败');
    }
  }, []);

  const remove = useCallback(async (id: string) => {
    try {
      await deleteClip(id);
      setClips((prev) => prev.filter((c) => c.id !== id));
    } catch (e) {
      setError(e instanceof Error ? e.message : '删除失败');
    }
  }, []);

  const paste = useCallback(async (id: string) => {
    try {
      await pasteClip(id);
    } catch (e) {
      setError(e instanceof Error ? e.message : '粘贴失败');
    }
  }, []);

  useEffect(() => {
    loadClips();
    let unlistenFn: (() => void) | null = null;

    (async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlistenFn = await listen<ClipSummary>('clip-created', () => {
          loadClips();
        });
      } catch {
        // Tauri event not available
      }
    })();

    return () => {
      unlistenFn?.();
    };
  }, [loadClips]);

  return {
    clips,
    loading,
    error,
    search,
    refresh: loadClips,
    toggleFav,
    remove,
    paste,
    filterType,
    setFilterType,
    loadClipsSilent,
  };
}
