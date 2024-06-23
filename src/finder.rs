use crate::mapping::{Mapping, Position};
use crate::mappings::Mappings;
use std::cell::Cell;
use std::cmp::Ordering;

type FinderState = (
    // generated pos of the last finding
    Position,
    // result index in mappings of the last finding
    usize,
);

/// `MappingFinder` is a helper struct for finding mappings within a [BorrowedSourceMap](crate::BorrowedSourceMap).
///
/// It is highly efficient for frequent mapping findings,
/// especially when traversing the source map in small increments (e.g., sequentially
/// finding mappings from start to finish in a source map for a minified file).
#[derive(Debug)]
pub struct MappingFinder<'a> {
    // last mapping found
    state: Cell<FinderState>,
    finder: MappingFinderImpl<'a>,
}

impl<'a> MappingFinder<'a> {
    pub(crate) fn new(mappings: &'a Mappings) -> Self {
        Self {
            state: Cell::new((
                // set to the max position at first
                Position::max(),
                // no need to minus 1 because it will be the upper bound for the first searching
                mappings.len(),
            )),
            finder: MappingFinderImpl::new(mappings),
        }
    }

    /// Finds the mapping for a given generated position.
    ///
    /// If an exact match is not found, this method returns the closest preceding mapping.
    /// If there are no preceding mappings, it returns `None`.
    pub fn find_mapping<P>(&self, pos: P) -> Option<Mapping>
    where
        P: Into<Position>,
    {
        self.finder.find(pos.into(), Some(&self.state))
    }
}

#[derive(Debug)]
pub(crate) struct MappingFinderImpl<'a> {
    mappings: &'a Mappings,
}

impl<'a> MappingFinderImpl<'a> {
    pub(crate) fn new(mappings: &'a Mappings) -> Self {
        Self { mappings }
    }

    #[inline]
    pub(crate) fn find(&self, pos: Position, state: Option<&Cell<FinderState>>) -> Option<Mapping> {
        match state {
            Some(state) => {
                let (last_pos, last_idx) = state.get();

                let should_use_linear_search =
                    pos.line == last_pos.line && pos.column.abs_diff(last_pos.column) <= 32;

                let ordering = last_pos.cmp(&pos);

                if ordering == Ordering::Less {
                    if should_use_linear_search {
                        self.find_by_linear_search_down_to(pos, last_idx + 1)
                    } else {
                        self.find_by_binary_search_down_to(pos, last_idx + 1)
                    }
                } else if ordering == Ordering::Greater {
                    if should_use_linear_search {
                        self.find_by_linear_search_up_to(pos, last_idx)
                    } else {
                        // this is the branch that initial state will enter
                        self.find_by_binary_search_up_to(pos, last_idx)
                    }
                } else {
                    Some(last_idx)
                }
                .map(|idx| {
                    // SAFETY: idx returned is guaranteed valid
                    let result = unsafe { self.mappings.get_unchecked(idx).clone() };
                    state.set((result.generated(), idx));
                    result
                })
            }
            None => self
                .find_by_binary_search_up_to(pos, self.mappings.len())
                .map(|idx| unsafe { self.mappings.get_unchecked(idx) }.clone()),
        }
    }

    fn find_by_linear_search_up_to(&self, pos: Position, max_idx: usize) -> Option<usize> {
        (0..max_idx).rev().position(|idx| {
            // SAFETY: idx from 0 to max_idx is obviously safe since the max_idx is calculated
            //   within mappings before
            unsafe { self.mappings.get_unchecked(idx) }
                .generated()
                .le(&pos)
        })
    }

    fn find_by_linear_search_down_to(&self, pos: Position, min_idx: usize) -> Option<usize> {
        for idx in min_idx..self.mappings.len() {
            // SAFETY: idx from min_idx to self.map.mappings.len() is obviously safe
            // since the min_idx is calculated within mappings before,
            // max(min_idx) is mappings.len(), which is guarded by min_idx..mappings.len()
            let ordering = unsafe { self.mappings.get_unchecked(idx) }
                .generated()
                .cmp(&pos);
            if ordering == Ordering::Less {
                continue;
            } else if ordering == Ordering::Equal {
                return Some(idx);
            } else if ordering == Ordering::Greater {
                return Some(idx - 1);
            }
        }
        Some(self.mappings.len() - 1)
    }

    fn find_by_binary_search_up_to(&self, pos: Position, max_idx: usize) -> Option<usize> {
        // SAFETY: ..max_idx is in valid index range since the max_idx is calculated
        //   within mappings before
        match unsafe { self.mappings.get_unchecked(..max_idx) }
            .binary_search_by_key(&pos, Mapping::generated)
        {
            Ok(idx) => Some(idx),
            Err(0) => None,
            Err(idx) => Some(idx - 1),
        }
    }

    fn find_by_binary_search_down_to(&self, pos: Position, min_idx: usize) -> Option<usize> {
        // SAFETY: min_idx.. is in valid index range since the min_idx is calculated
        //   within mappings before, and always be > 0
        Some(
            match unsafe { self.mappings.get_unchecked(min_idx..) }
                .binary_search_by_key(&pos, Mapping::generated)
            {
                Ok(idx) => min_idx + idx,
                Err(idx) => min_idx + idx - 1,
            },
        )
    }
}
