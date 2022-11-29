// Logic to determine if a sparse grid should become a DenseGrid

use crate::grid::CellCoordinate;
use std::fmt::Debug;

///
/// Determines if given a iteratator of CellCoordinate if we should be sparse or not.
pub trait ConvertHeuristic: Debug {
    fn convert_to_sparse<'a>(&'a self, it: Box<dyn Iterator<Item = CellCoordinate> + 'a>) -> bool;
}

///
/// Heuristic that looks at the max gap between rows or columns.
#[derive(Debug)]
pub struct MaxGapConvertHeuristic {
    max_gap_threshold: usize,
}

impl MaxGapConvertHeuristic {
    pub fn new(max_gap_threshold: usize) -> Self {
        Self { max_gap_threshold }
    }
}

impl ConvertHeuristic for MaxGapConvertHeuristic {
    fn convert_to_sparse<'a>(&'a self, it: Box<dyn Iterator<Item = CellCoordinate> + 'a>) -> bool {
        let mut row_gap_measurment = GapMeasurement::default();
        let mut col_gap_measurment = GapMeasurement::default();
        for coord in it {
            row_gap_measurment.add(coord.row);
            col_gap_measurment.add(coord.col);
        }

        return row_gap_measurment.max_gap >= self.max_gap_threshold
            || col_gap_measurment.max_gap >= self.max_gap_threshold;
    }
}

///
/// Measures the gap between a sequence of numbers
#[derive(Default, Debug)]
struct GapMeasurement {
    gaps: Vec<usize>,
    max_gap: usize,
}
impl GapMeasurement {
    // add a `value` in this case a coordinate and update the max gap seen
    fn add(&mut self, value: usize) {
        // check_pos is the position that the idx is, we need to check the gap on either side of it.
        let check_pos = match self.gaps.binary_search(&value) {
            Ok(_) => {
                //already filled, keep moving
                return;
            }
            Err(pos) => {
                self.gaps.insert(pos, value);
                pos
            }
        };

        if check_pos > 0 {
            //check the gap on the lower side
            let lower_neighbor = self.gaps[check_pos - 1];
            let gap = value - lower_neighbor;
            self.max_gap = self.max_gap.max(gap);
        }

        if check_pos + 1 != self.gaps.len() {
            // check for the gap agains our higher neighbor
            let higher_neighbor = self.gaps[check_pos + 1];
            let gap = higher_neighbor - value;
            self.max_gap = self.max_gap.max(gap);
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;

    fn gap_measurment_fixture(elements: Vec<usize>, expected_gap: usize) {
        let mut gap_measurment = GapMeasurement::default();
        for element in elements {
            gap_measurment.add(element);
        }

        assert_eq!(gap_measurment.max_gap, expected_gap);
    }

    #[test]
    fn gap_measurment_1() {
        gap_measurment_fixture(vec![1], 0);
    }

    #[test]
    fn gap_measurment_2() {
        gap_measurment_fixture(vec![1, 10], 9);
    }
    #[test]
    fn gap_measurment_ooo() {
        gap_measurment_fixture(vec![10, 5, 1], 5);
    }
}
