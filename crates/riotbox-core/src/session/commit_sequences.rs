//! Derived per-boundary capacity, not a persisted cursor or exclusive reservation.
use std::{error::Error, fmt, ops::RangeInclusive};

use crate::transport::CommitBoundaryState;

use super::ActionLog;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitSequenceExhausted {
    pub boundary: CommitBoundaryState,
    pub requested_count: usize,
    pub max_existing: u32,
}

impl fmt::Display for CommitSequenceExhausted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "commit sequence exhausted for {:?} boundary: {} requested after {}",
            self.boundary, self.requested_count, self.max_existing
        )
    }
}

impl Error for CommitSequenceExhausted {}

/// Checked absolute sequences for one intended operation.
///
/// This is arithmetic evidence only. Writers must check current history again
/// when they mutate queue state; a range does not reserve space in an ActionLog.
#[derive(Debug)]
pub struct CommitSequenceRange {
    remaining: Option<RangeInclusive<u32>>,
}

impl Iterator for CommitSequenceRange {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.remaining.as_mut().and_then(Iterator::next)
    }
}

impl ActionLog {
    pub fn prepare_commit_sequences(
        &self,
        boundary: &CommitBoundaryState,
        count: usize,
    ) -> Result<CommitSequenceRange, CommitSequenceExhausted> {
        if count == 0 {
            return Ok(CommitSequenceRange { remaining: None });
        }

        let max_existing = self
            .commit_records
            .iter()
            .filter(|record| record.boundary == *boundary)
            .map(|record| record.commit_sequence)
            .max()
            .unwrap_or(0);
        let exhausted = || CommitSequenceExhausted {
            boundary: boundary.clone(),
            requested_count: count,
            max_existing,
        };
        let count = u32::try_from(count).map_err(|_| exhausted())?;
        let last = max_existing.checked_add(count).ok_or_else(exhausted)?;
        let first = max_existing.checked_add(1).ok_or_else(exhausted)?;
        Ok(CommitSequenceRange {
            remaining: Some(first..=last),
        })
    }
}

#[cfg(test)]
mod tests;
