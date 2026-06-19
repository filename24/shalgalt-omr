/**
 * Editor selection store.
 *
 * Tracks which bubble group (if any) is currently selected. Markers are not
 * selectable — they always render in fixed positions per their TL/TR/BR/BL
 * role. The LayerTree, Inspector, and BubbleGroupNode components all read and
 * mutate this single store so their UIs stay in sync.
 */
class EditorSelectionStore {
  selectedGroupId = $state<string | null>(null);

  select(id: string | null): void {
    this.selectedGroupId = id;
  }

  clear(): void {
    this.selectedGroupId = null;
  }
}

export const editorSelection = new EditorSelectionStore();
