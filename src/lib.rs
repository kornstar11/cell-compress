mod distance;
mod grid;

pub use distance::{ConvertHeuristic, MaxGapConvertHeuristic};
pub use grid::{CellCoordinate, GridCell, GridContainer};

//
// Library that repersents a grid/matrix of data, and tries to keep it in the most efficient reperesentation at all times
#[cfg(test)]
mod test {
    use crate::grid::Grid;

    use super::*;

    #[test]
    fn grid_container_converts() {
        // Setting max_gap_threshold to 5 means on the 2nd element our gap should be 10 and we should convert to the sparse repersentation
        let mut g = GridContainer::new(Box::new(MaxGapConvertHeuristic::new(5)));

        assert_eq!(g.is_sparse(), false);

        let coord1 = CellCoordinate { col: 0, row: 0 };
        let coord2 = CellCoordinate { col: 10, row: 10 };
        g.insert(&coord1, GridCell::new("data 1"));
        assert_eq!(g.is_sparse(), false);
        g.insert(&coord2, GridCell::new("data 2"));
        assert_eq!(g.is_sparse(), true);
    }
}
