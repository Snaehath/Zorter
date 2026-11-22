import type { ReactNode } from 'react';
import { FolderOpen } from 'lucide-react';
import { useStore } from '../lib/storeContext';
import { Button } from './ui';

/** Current working folder + picker, shared by Scan / Analyze / Organize. */
export function FolderBar({ disabled, children }: { disabled?: boolean; children?: ReactNode }) {
  const { folder, chooseFolder } = useStore();
  return (
    <div className="mb-5 flex items-center gap-3 rounded-xl border border-line bg-panel px-4 py-3">
      <FolderOpen size={18} className="shrink-0 text-accent" />
      <div className="min-w-0 flex-1">
        <div className="text-[11px] tracking-wider text-faint">FOLDER</div>
        <div className="truncate text-[13px]" title={folder ?? undefined}>
          {folder ?? <span className="text-dim">No folder selected</span>}
        </div>
      </div>
      <Button onClick={() => void chooseFolder()} disabled={disabled}>
        {folder ? 'Change' : 'Choose folder'}
      </Button>
      {children}
    </div>
  );
}
