import { useState } from 'react';
import { ChevronDown, FileText, Folder, RotateCcw, SlidersHorizontal, TriangleExclamationPoint, type LucideIcon } from 'lucide-react';
import { DEFAULT_OPTIONS, LAYOUTS, NAMINGS, UNANALYZED, type Choice } from '../lib/planOptions';
import type { PlanOptions } from '../lib/types';
import { cx } from '../lib/cx';
import { Panel } from './ui';

function Group<T extends string>({
  icon: Icon,
  title,
  choices,
  value,
  root,
  onChange,
}: {
  icon: LucideIcon;
  title: string;
  choices: Choice<T>[];
  value: T;
  root: string;
  onChange: (v: T) => void;
}) {
  return (
    <fieldset>
      <legend className="mb-2 flex items-center gap-2 text-[12px] font-semibold tracking-wider text-accent">
        <Icon size={14} /> {title.toUpperCase()}
      </legend>
      <div className="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-2">
        {choices.map((c) => {
          const on = c.value === value;
          return (
            <label
              key={c.value}
              className={cx(
                'cursor-pointer rounded-lg border px-3 py-2 transition-colors',
                on ? 'border-accent bg-accent/10' : 'border-line hover:border-accent/50 hover:bg-raised',
              )}
            >
              <input type="radio" className="sr-only" checked={on} onChange={() => onChange(c.value)} />
              <div className={cx('text-[13px]', on ? 'text-text' : 'text-dim')}>{c.label}</div>
              <div className="mt-0.5 truncate text-[11px] text-faint">{c.example.replace('{root}', root)}</div>
            </label>
          );
        })}
      </div>
    </fieldset>
  );
}

/** Collapsible "how to organize" options for the Organize screen. */
export function OrganizeOptions({
  value,
  root,
  disabled,
  onChange,
}: {
  value: Required<PlanOptions>;
  root: string;
  disabled?: boolean;
  onChange: (o: Required<PlanOptions>) => void;
}) {
  const [open, setOpen] = useState(false);
  const layout = LAYOUTS.find((c) => c.value === value.layout)!;
  const naming = NAMINGS.find((c) => c.value === value.naming)!;
  const isDefault =
    value.layout === DEFAULT_OPTIONS.layout && value.naming === DEFAULT_OPTIONS.naming && value.move_unanalyzed === DEFAULT_OPTIONS.move_unanalyzed;

  return (
    <Panel className={cx('mb-5', disabled && 'pointer-events-none opacity-60')}>
      <button
        onClick={() => setOpen((o) => !o)}
        aria-expanded={open}
        className="flex w-full items-center gap-3 px-4 py-3 text-left text-[13px]"
      >
        <SlidersHorizontal size={16} className="text-accent" />
        <span className="flex min-w-0 flex-1 flex-wrap gap-x-4 gap-y-1">
          <span>
            <span className="text-dim">Folders:</span> {layout.label}
          </span>
          <span>
            <span className="text-dim">Names:</span> {naming.label}
          </span>
          <span>
            <span className="text-dim">Unanalyzed:</span> {value.move_unanalyzed ? 'Needs-Review/' : 'left in place'}
          </span>
        </span>
        <span className="text-[12px] text-accent">{open ? 'Done' : 'Customize'}</span>
        <ChevronDown size={16} className={cx('text-dim transition-transform', open && 'rotate-180')} />
      </button>

      {open && (
        <div className="animate-in space-y-5 border-t border-line-soft px-4 pb-4 pt-4">
          <Group icon={Folder} title="Folders" choices={LAYOUTS} value={value.layout} root={root} onChange={(layout) => onChange({ ...value, layout })} />
          <Group icon={FileText} title="File names" choices={NAMINGS} value={value.naming} root={root} onChange={(naming) => onChange({ ...value, naming })} />
          <Group
            icon={TriangleExclamationPoint}
            title="Files not analyzed yet"
            choices={UNANALYZED}
            value={value.move_unanalyzed ? 'move' : 'leave'}
            root={root}
            onChange={(v) => onChange({ ...value, move_unanalyzed: v === 'move' })}
          />
          {!isDefault && (
            <button onClick={() => onChange(DEFAULT_OPTIONS)} className="flex items-center gap-1.5 text-[12px] text-dim hover:text-text">
              <RotateCcw size={13} /> Reset to defaults
            </button>
          )}
        </div>
      )}
    </Panel>
  );
}
