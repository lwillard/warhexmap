use crate::hex::map::{HexCell, HexMap};

#[derive(Clone)]
pub enum UndoAction {
    SetCell { q: i32, r: i32, old: HexCell },
    BatchSetCells(Vec<(i32, i32, HexCell)>),
}

pub struct UndoStack {
    pub undo: Vec<UndoAction>,
    pub redo: Vec<UndoAction>,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    pub fn push(&mut self, action: UndoAction) {
        self.undo.push(action);
        self.redo.clear();
    }

    pub fn undo(&mut self, map: &mut HexMap) {
        if let Some(action) = self.undo.pop() {
            match &action {
                UndoAction::SetCell { q, r, old } => {
                    // Save current state for redo before restoring
                    if let Some(current) = map.cells.get(&(*q, *r)).cloned() {
                        self.redo.push(UndoAction::SetCell {
                            q: *q,
                            r: *r,
                            old: current,
                        });
                    }
                    map.cells.insert((*q, *r), old.clone());
                }
                UndoAction::BatchSetCells(batch) => {
                    let mut redo_batch = Vec::new();
                    for (q, r, old) in batch {
                        if let Some(current) = map.cells.get(&(*q, *r)).cloned() {
                            redo_batch.push((*q, *r, current));
                        }
                        map.cells.insert((*q, *r), old.clone());
                    }
                    self.redo.push(UndoAction::BatchSetCells(redo_batch));
                }
            }
        }
    }

    pub fn redo(&mut self, map: &mut HexMap) {
        if let Some(action) = self.redo.pop() {
            match &action {
                UndoAction::SetCell { q, r, old } => {
                    if let Some(current) = map.cells.get(&(*q, *r)).cloned() {
                        self.undo.push(UndoAction::SetCell {
                            q: *q,
                            r: *r,
                            old: current,
                        });
                    }
                    map.cells.insert((*q, *r), old.clone());
                }
                UndoAction::BatchSetCells(batch) => {
                    let mut undo_batch = Vec::new();
                    for (q, r, old) in batch {
                        if let Some(current) = map.cells.get(&(*q, *r)).cloned() {
                            undo_batch.push((*q, *r, current));
                        }
                        map.cells.insert((*q, *r), old.clone());
                    }
                    self.undo.push(UndoAction::BatchSetCells(undo_batch));
                }
            }
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}
