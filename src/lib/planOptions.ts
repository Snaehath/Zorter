import type { PlanOptions } from './types';

export interface Choice<T extends string> {
  value: T;
  label: string;
  /** Example of the result; `{root}` is replaced with the folder name. */
  example: string;
}

export type Layout = 'by_category' | 'category' | 'category_and_style' | 'in_place' | 'flat_rename';
export type Naming = 'subject_scene' | 'kebab-case' | 'title_case' | 'detailed' | 'ai_suggested' | 'original';

export const LAYOUTS: Choice<Layout>[] = [
  { value: 'by_category', label: 'Category / Subcategory', example: '{root}/Nature/Forest/' },
  { value: 'category', label: 'Category only', example: '{root}/Nature/' },
  { value: 'category_and_style', label: 'Category / Style', example: '{root}/Nature/Minimalist/' },
  { value: 'in_place', label: 'Keep current folders', example: 'No new folders — rename only' },
  { value: 'flat_rename', label: 'One folder', example: 'Everything into {root}/' },
];

export const NAMINGS: Choice<Naming>[] = [
  { value: 'subject_scene', label: 'snake_case', example: 'forest_sunrise.jpg' },
  { value: 'kebab-case', label: 'kebab-case', example: 'forest-sunrise.jpg' },
  { value: 'title_case', label: 'Title_Case', example: 'Forest_Sunrise.jpg' },
  { value: 'detailed', label: 'Detailed', example: 'forest_sunrise_misty_morning.jpg' },
  { value: 'ai_suggested', label: 'AI suggested', example: "The model's own name" },
  { value: 'original', label: 'Keep original names', example: 'IMG_0042.jpg — sort only' },
];

export const UNANALYZED: Choice<'move' | 'leave'>[] = [
  { value: 'move', label: 'Move to Needs-Review/', example: 'Collect them for manual sorting' },
  { value: 'leave', label: 'Leave where they are', example: 'Only analyzed files change' },
];

export const DEFAULT_OPTIONS: Required<PlanOptions> = {
  layout: 'by_category',
  naming: 'subject_scene',
  move_unanalyzed: true,
};

const KEY = 'zorter.planOptions';

export function loadOptions(): Required<PlanOptions> {
  try {
    const saved = JSON.parse(localStorage.getItem(KEY) ?? '{}') as PlanOptions;
    return {
      layout: LAYOUTS.some((c) => c.value === saved.layout) ? saved.layout! : DEFAULT_OPTIONS.layout,
      naming: NAMINGS.some((c) => c.value === saved.naming) ? saved.naming! : DEFAULT_OPTIONS.naming,
      move_unanalyzed: typeof saved.move_unanalyzed === 'boolean' ? saved.move_unanalyzed : DEFAULT_OPTIONS.move_unanalyzed,
    };
  } catch {
    return DEFAULT_OPTIONS;
  }
}

export function saveOptions(o: Required<PlanOptions>): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(o));
  } catch {
    /* storage unavailable: options just aren't remembered */
  }
}
