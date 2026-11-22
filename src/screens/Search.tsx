import { useEffect, useRef, useState, type FormEvent } from 'react';
import { Box, ExternalLink, Folder, Images, Search, TriangleExclamationPoint } from 'lucide-react';
import { api, errorText } from '../lib/api';
import type { SearchResponse } from '../lib/types';
import { Button, Heading, Notice, Panel, Thumb } from '../components/ui';
import { cx } from '../lib/cx';

const PAGE = 6;

export function SearchScreen() {
  const [query, setQuery] = useState('');
  const [result, setResult] = useState<SearchResponse | null>(null);
  const [limit, setLimit] = useState(PAGE);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const lastQuery = useRef('');
  const input = useRef<HTMLInputElement>(null);

  useEffect(() => input.current?.focus(), []);

  const run = async (q: string, n: number) => {
    if (!q.trim()) return;
    setBusy(true);
    setError(null);
    try {
      setResult(await api.search(q.trim(), n));
      lastQuery.current = q.trim();
      setLimit(n);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  const submit = (e: FormEvent) => {
    e.preventDefault();
    void run(query, PAGE);
  };

  return (
    <div className="animate-in">
      <Heading icon={Search} title="Semantic search" />

      <form onSubmit={submit} className="mb-5 flex gap-3">
        <div className="flex flex-1 items-center gap-3 rounded-xl border border-line bg-panel px-4 focus-within:border-accent/70">
          <Search size={18} className="text-dim" />
          <input
            ref={input}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Describe what you're looking for, e.g. dark cyberpunk city wallpapers"
            className="h-12 flex-1 bg-transparent text-[14px] outline-none placeholder:text-faint"
            aria-label="Search query"
          />
        </div>
        <Button variant="primary" type="submit" icon={Search} busy={busy} disabled={!query.trim()} className="h-12 px-5">
          Search
        </Button>
      </form>

      {error && <Notice tone="bad" title="Search failed">{error}</Notice>}

      {result && (
        <>
          <div className="mb-3 flex items-center justify-between border-b border-line-soft pb-3 text-[13px]">
            {result.mode === 'semantic' ? (
              <span className="flex items-center gap-2 text-dim">
                <Box size={15} /> Embedding: <span className="text-accent">{result.model}</span>
              </span>
            ) : (
              <span className="flex items-center gap-2 text-warn" title={result.reason ?? ''}>
                <TriangleExclamationPoint size={15} /> Keyword match — {result.reason}
              </span>
            )}
            <span className="flex items-center gap-2 text-dim">
              <Images size={15} /> Results:{' '}
              <span className="text-text">
                {result.total} match{result.total === 1 ? '' : 'es'}
              </span>
            </span>
          </div>

          {result.hits.length === 0 ? (
            <Panel className="px-6 py-10 text-center text-dim">No matching files. Try different words.</Panel>
          ) : (
            <div className="space-y-3">
              {result.hits.map((h) => (
                <Panel key={h.path} tone={h.rank === 1 ? 'ok' : 'default'} className="animate-in">
                  <div className="flex items-center gap-4 px-4 py-3">
                    <span className="w-7 text-center text-[16px] font-semibold text-dim">{String(h.rank).padStart(2, '0')}</span>
                    <Thumb path={h.path} hash={h.hash} className="h-[68px] w-[110px] shrink-0" />
                    <div className="min-w-0 flex-1">
                      <div className="truncate font-semibold" title={h.name}>
                        {h.name}
                      </div>
                      <div className="mt-0.5 flex items-center gap-1.5 text-[12px] text-dim">
                        <Folder size={13} /> <span className="truncate">{h.folder}</span>
                      </div>
                      {h.description && <div className="mt-1 truncate text-[12px] text-faint">{h.description}</div>}
                    </div>
                    <div className="text-right">
                      <div className="text-[11px] text-dim">{result.mode === 'semantic' ? 'Similarity' : 'Match'}</div>
                      <div
                        className={cx(
                          'text-[20px] font-semibold',
                          h.score >= 0.75 ? 'text-ok' : h.score >= 0.5 ? 'text-accent' : 'text-dim',
                        )}
                      >
                        {h.score.toFixed(2)}
                      </div>
                    </div>
                    <button
                      onClick={() => void api.reveal(h.path)}
                      className="rounded-lg p-2 text-dim hover:bg-raised hover:text-text"
                      title="Show in folder"
                      aria-label={`Show ${h.name} in folder`}
                    >
                      <ExternalLink size={16} />
                    </button>
                  </div>
                </Panel>
              ))}
            </div>
          )}

          {result.hits.length > 0 && (
            <div className="mt-4 flex items-center justify-between text-[13px] text-dim">
              <span>
                Showing {result.hits.length} of {result.total} results
              </span>
              {result.total > result.hits.length && (
                <Button onClick={() => void run(lastQuery.current, limit + PAGE)} busy={busy}>
                  Show more
                </Button>
              )}
            </div>
          )}
        </>
      )}

      {!result && !error && (
        <Panel className="px-6 py-10 text-center text-dim">
          Search your analyzed images by meaning — colours, mood, subjects, places.
        </Panel>
      )}
    </div>
  );
}
