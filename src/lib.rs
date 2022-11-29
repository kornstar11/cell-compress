use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridCell {
    data: String,
    //todo GridCellAttributes for things like board, font, etc?
}

impl GridCell {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        GridCell { data: s.as_ref().to_string() }

    }

    
}

// CellCoordinate: Position of a cell in the grid to be rendered
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct CellCoordinate {
    row: usize,
    col: usize,
}
/// 
/// Repersents the entire collection of cells. Depending on the state of the grid, distance between cells we may use a Sparse or Dense impl.
/// This change is handled in the GridContainer

pub trait Grid {
    // might make sense to add a "entry()" call at some point.
    fn get_mut(&mut self, coord: &CellCoordinate) -> Option<&mut GridCell>;
    fn remove(&mut self, coord: &CellCoordinate) -> Option<GridCell>;
    fn insert(&mut self, coord: &CellCoordinate, cell: GridCell);
    
}
/// Repersenets the case where most of the grid is full of gaps (distance between cells). For instance: there is a occupied cell at (1, 1) and (1, 100) 
#[derive(Clone, Default, Debug)]
pub struct SparseGrid {
    cols_rows: HashMap<CellCoordinate, GridCell>
}

impl Grid for SparseGrid {
    fn get_mut(&mut self, coord: &CellCoordinate) -> Option<&mut GridCell> {
        self.cols_rows.get_mut(coord)
    }

    fn remove(&mut self, coord: &CellCoordinate) -> Option<GridCell> {
        self.cols_rows.remove(coord)
    }

    fn insert(&mut self, coord: &CellCoordinate, cell: GridCell) {
        self.cols_rows.insert(*coord, cell);
    }
}

impl Into<DenseGrid> for SparseGrid {
    fn into(self) -> DenseGrid {
        let mut dense = DenseGrid::default();
        for (coord, cell) in self.cols_rows.into_iter() {
            dense.insert(&coord, cell);
        }
        dense
    }
}



/// Repersents the case where the Grid is close to full, we use Option<Cell> here to avoid the need to re-shuffle the vecs if an item is removed
#[derive(Clone, Default, Debug)]
pub struct DenseGrid {
    cols_rows: Vec<Vec<Option<GridCell>>>
}

impl Grid for DenseGrid {
    fn get_mut(&mut self, coord: &CellCoordinate) -> Option<&mut GridCell> {
        if let Some(rows) = self.cols_rows.get_mut(coord.col) {
            return rows.get_mut(coord.row).and_then(|ident| ident.as_mut());
        } 
        return None;
        
    }

    fn remove(&mut self, coord: &CellCoordinate) -> Option<GridCell> {
        if let Some(rows) = self.cols_rows.get_mut(coord.col) {
            if let Some(cell_ref) = rows.get_mut(coord.row) {
                return std::mem::take(cell_ref);
            }
            return None;
        } 
        return None;
    }

    fn insert(&mut self, coord: &CellCoordinate, cell: GridCell) {
        if coord.col >= self.cols_rows.len() {
            self.cols_rows.resize_with(coord.col + 1, || {vec![]});
        }
        if let Some(rows) = self.cols_rows.get_mut(coord.col) {
            if coord.row >= rows.len() {
                rows.resize_with(coord.row + 1, || None);
            }

            if let Some(cell_ref) = rows.get_mut(coord.row) {
                *cell_ref = Some(cell);
            } else {
                rows.insert(coord.row, Some(cell));
            }
        }
        // don't worry about the else case since we should not hit it if we re-size correctly
    }
}

impl Into<SparseGrid> for DenseGrid {
    fn into(self) -> SparseGrid {
        let mut sparse = SparseGrid::default();
        for (col_idx, rows) in self.cols_rows.into_iter().enumerate() {
            for (row_idx, cell_opt) in rows.into_iter().enumerate() {
                if let Some(cell) = cell_opt {
                    let coord = CellCoordinate {
                        col:col_idx,
                        row: row_idx,
                    };
                    sparse.insert(&coord, cell);
                }
            }
        }

        sparse
    }
}

/// 
/// Idea for compessing

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    fn can_insert_and_get_and_remove<G: Grid + Debug>(mut g: G) {
        let coord1 = CellCoordinate{col:0, row:0};
        let coord2 = CellCoordinate{col:10, row:10};
        g.insert(&coord1, GridCell::new("data 1"));
        g.insert(&coord2, GridCell::new("data 2"));

        println!("Grid state: {:?}", g);

        assert_eq!(g.get_mut(&coord1).cloned(), Some(GridCell{data: "data 1".to_string()}));
        assert_eq!(g.get_mut(&coord2).cloned(), Some(GridCell{data: "data 2".to_string()}));

        g.remove(&coord1);
        assert_eq!(g.get_mut(&coord1).cloned(), None);

        g.remove(&coord2);
        assert_eq!(g.get_mut(&coord2).cloned(), None);
    }

    #[test]
    fn dense_can_insert_and_get() {
        can_insert_and_get_and_remove(DenseGrid::default());
    }

    #[test]
    fn sparse_can_insert_and_get() {
        can_insert_and_get_and_remove(SparseGrid::default());
    }
}
