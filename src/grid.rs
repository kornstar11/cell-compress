use std::{collections::HashMap, fmt::Debug};

use crate::distance::ConvertHeuristic;

///
/// Module repersents a Grid of cells (maybe matrix). The cells sort a small amount of information, such as a formula or value. To keep things memory effiecent and speedy, we try to swap between dense and sparse
/// repersentations of the Grid. The SparseGrid, is backed by a HashMap, while the DenseGrid is backend by a Vec<Vec<Cell>>. To make a determination on if we should be dense or sparse we use a `ConvertHeuristic`
/// that will indicate if the grid should be sparse.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridCell {
    data: String,
    //todo GridCellAttributes for things like board, font, etc?
}

impl GridCell {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        GridCell {
            data: s.as_ref().to_string(),
        }
    }
}

// CellCoordinate: Position of a cell in the grid to be rendered
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CellCoordinate {
    pub row: usize,
    pub col: usize,
}
///
/// Repersents the entire collection of cells. Depending on the state of the grid, distance between cells we may use a Sparse or Dense impl.
/// This change is handled in the GridContainer
pub trait Grid: Debug {
    // might make sense to add a "entry()" call at some point.
    fn get_mut(&mut self, coord: &CellCoordinate) -> Option<&mut GridCell>;
    fn remove(&mut self, coord: &CellCoordinate) -> Option<GridCell>;
    fn insert(&mut self, coord: &CellCoordinate, cell: GridCell);

    //fn coord_iter(&self) -> Box<dyn Iterator<Item = CellCoordinate>>;
    fn coord_iter<'a>(&'a self) -> Box<dyn Iterator<Item = CellCoordinate> + 'a>;
}

trait Convertable {
    fn should_convert(&self) -> bool;
}

/// Repersenets the case where most of the grid is full of gaps (distance between cells). For instance: there is a occupied cell at (1, 1) and (1, 100)
#[derive(Clone, Default, Debug)]
struct SparseGrid {
    cols_rows: HashMap<CellCoordinate, GridCell>,
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
    fn coord_iter<'a>(&'a self) -> Box<dyn Iterator<Item = CellCoordinate> + 'a> {
        Box::new(self.cols_rows.keys().map(|e| e.clone()))
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
struct DenseGrid {
    cols_rows: Vec<Vec<Option<GridCell>>>,
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
            self.cols_rows.resize_with(coord.col + 1, || vec![]);
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

    fn coord_iter<'a>(&'a self) -> Box<dyn Iterator<Item = CellCoordinate> + 'a> {
        let iter = self
            .cols_rows
            .iter()
            .enumerate()
            .flat_map(|(col_idx, rows)| {
                let row_iter = Box::new(rows.iter().enumerate());
                DenseGridIter { col_idx, row_iter }
            })
            .flat_map(|opt| opt.into_iter());
        Box::new(iter)
    }
}

struct DenseGridIter<I> {
    col_idx: usize,
    row_iter: I,
}

impl<'a, I: Iterator<Item = (usize, &'a Option<GridCell>)>> Iterator for DenseGridIter<I> {
    type Item = Option<CellCoordinate>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.row_iter.next() {
            Some((row_idx, Some(_))) => Some(Some(CellCoordinate {
                col: self.col_idx,
                row: row_idx,
            })),
            Some((_, None)) => Some(None),
            _ => None,
        }
    }
}

impl Into<SparseGrid> for DenseGrid {
    fn into(self) -> SparseGrid {
        let mut sparse = SparseGrid::default();
        for (col_idx, rows) in self.cols_rows.into_iter().enumerate() {
            for (row_idx, cell_opt) in rows.into_iter().enumerate() {
                if let Some(cell) = cell_opt {
                    let coord = CellCoordinate {
                        col: col_idx,
                        row: row_idx,
                    };
                    sparse.insert(&coord, cell);
                }
            }
        }

        sparse
    }
}

#[derive(Clone, Debug)]
enum SwappingGrid {
    Dense(DenseGrid),
    Sparse(SparseGrid),
}

impl SwappingGrid {
    fn swap(self) -> Self {
        match self {
            Self::Dense(g) => Self::Sparse(g.into()),
            Self::Sparse(g) => Self::Dense(g.into()),
        }
    }

    fn default() -> Self {
        Self::Dense(DenseGrid::default())
    }
}

///
/// GridContainer: handles swapping between different Grid impls
#[derive(Debug)]
pub struct GridContainer {
    grid: SwappingGrid,
    convert_heuristic: Box<dyn ConvertHeuristic>,
}

impl GridContainer {
    pub fn new(convert_heuristic: Box<dyn ConvertHeuristic>) -> Self {
        Self {
            grid: SwappingGrid::default(),
            convert_heuristic,
        }
    }

    pub fn is_sparse(&self) -> bool {
        match self.grid {
            SwappingGrid::Sparse(_) => true,
            _ => false,
        }
    }
}

impl Grid for GridContainer {
    fn get_mut(&mut self, coord: &CellCoordinate) -> Option<&mut GridCell> {
        match self.grid {
            SwappingGrid::Dense(ref mut g) => g.get_mut(coord),
            SwappingGrid::Sparse(ref mut g) => g.get_mut(coord),
        }
    }

    fn remove(&mut self, coord: &CellCoordinate) -> Option<GridCell> {
        let (return_opt, should_be_sparse, is_sparse) = match self.grid {
            SwappingGrid::Dense(ref mut g) => {
                let ret = g.remove(coord);
                let it = g.coord_iter();
                let convert = self.convert_heuristic.convert_to_sparse(it);
                (ret, convert, false)
            }
            SwappingGrid::Sparse(ref mut g) => {
                let ret = g.remove(coord);
                let it = g.coord_iter();
                let convert = self.convert_heuristic.convert_to_sparse(it);
                (ret, convert, true)
            }
        };

        if (should_be_sparse && !is_sparse) || (!should_be_sparse && is_sparse) {
            let to_swap = std::mem::replace(&mut self.grid, SwappingGrid::default());
            self.grid = to_swap.swap();
        }

        return return_opt;
    }

    fn insert(&mut self, coord: &CellCoordinate, cell: GridCell) {
        let (should_be_sparse, is_sparse) = match self.grid {
            SwappingGrid::Dense(ref mut g) => {
                g.insert(coord, cell);
                let it = g.coord_iter();
                let convert = self.convert_heuristic.convert_to_sparse(it);
                (convert, false)
            }
            SwappingGrid::Sparse(ref mut g) => {
                g.insert(coord, cell);
                let it = g.coord_iter();
                let convert = self.convert_heuristic.convert_to_sparse(it);
                (convert, true)
            }
        };

        if (should_be_sparse && !is_sparse) || (!should_be_sparse && is_sparse) {
            let to_swap = std::mem::replace(&mut self.grid, SwappingGrid::default());
            self.grid = to_swap.swap();
        }
    }

    fn coord_iter<'a>(&'a self) -> Box<dyn Iterator<Item = CellCoordinate> + 'a> {
        match self.grid {
            SwappingGrid::Dense(ref g) => g.coord_iter(),
            SwappingGrid::Sparse(ref g) => g.coord_iter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    fn can_insert_and_get_and_remove<G: Grid + Debug>(mut g: G) {
        let coord1 = CellCoordinate { col: 0, row: 0 };
        let coord2 = CellCoordinate { col: 10, row: 10 };
        g.insert(&coord1, GridCell::new("data 1"));
        g.insert(&coord2, GridCell::new("data 2"));

        println!("Grid state: {:?}", g);

        assert_eq!(
            g.get_mut(&coord1).cloned(),
            Some(GridCell {
                data: "data 1".to_string()
            })
        );
        assert_eq!(
            g.get_mut(&coord2).cloned(),
            Some(GridCell {
                data: "data 2".to_string()
            })
        );

        g.remove(&coord1);
        assert_eq!(g.get_mut(&coord1).cloned(), None);

        g.remove(&coord2);
        assert_eq!(g.get_mut(&coord2).cloned(), None);
    }

    fn can_iter<G: Grid + Debug>(mut g: G) {
        let coord1 = CellCoordinate { col: 0, row: 0 };
        let coord2 = CellCoordinate { col: 10, row: 10 };
        g.insert(&coord1, GridCell::new("data 1"));
        g.insert(&coord2, GridCell::new("data 2"));

        let mut coords = g.coord_iter().collect::<Vec<_>>();
        coords.sort(); // hashmap of sparse does not give us order
        assert_eq!(
            coords,
            vec![
                CellCoordinate { row: 0, col: 0 },
                CellCoordinate { row: 10, col: 10 }
            ]
        )
    }

    #[test]
    fn dense_can_insert_and_get() {
        can_insert_and_get_and_remove(DenseGrid::default());
    }

    #[test]
    fn sparse_can_insert_and_get() {
        can_insert_and_get_and_remove(SparseGrid::default());
    }

    #[test]
    fn sparse_can_iter() {
        can_iter(SparseGrid::default())
    }
    #[test]
    fn dense_can_iter() {
        can_iter(DenseGrid::default())
    }
}
