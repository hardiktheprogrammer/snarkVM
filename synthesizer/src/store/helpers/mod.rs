// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

pub mod memory;
#[cfg(feature = "rocks")]
pub mod rocksdb;

use console::network::prelude::*;

use core::{borrow::Borrow, hash::Hash};
use std::borrow::Cow;

/// A trait representing map-like storage operations with read-write capabilities.
pub trait Map<
    'a,
    K: 'a + Copy + Clone + PartialEq + Eq + Hash + Serialize + Deserialize<'a> + Send + Sync,
    V: 'a + Clone + PartialEq + Eq + Serialize + Deserialize<'a> + Send + Sync,
>: Clone + MapRead<'a, K, V> + Send + Sync
{
    ///
    /// Inserts the given key-value pair into the map.
    ///
    fn insert(&self, key: K, value: V) -> Result<()>;

    ///
    /// Removes the key-value pair for the given key from the map.
    ///
    fn remove(&self, key: &K) -> Result<()>;

    ///
    /// Begins an atomic operation. Any further calls to `insert` and `remove` will be queued
    /// without an actual write taking place until `finish_atomic` is called.
    ///
    fn start_atomic(&self);

    ///
    /// Checks whether an atomic operation is currently in progress. This can be done to ensure
    /// that lower-level operations don't start or finish their individual atomic write batch
    /// if they are already part of a larger one.
    ///
    fn is_atomic_in_progress(&self) -> bool;

    ///
    /// Saves the current list of pending operations, so that if `atomic_rewind` is called,
    /// we roll back all future operations, and return to the start of this checkpoint.
    ///
    fn atomic_checkpoint(&self);

    ///
    /// Removes all pending operations to the last `atomic_checkpoint`
    /// (or to `start_atomic` if no checkpoints have been created).
    ///
    fn atomic_rewind(&self);

    ///
    /// Aborts the current atomic operation.
    ///
    fn abort_atomic(&self);

    ///
    /// Finishes an atomic operation, performing all the queued writes.
    ///
    fn finish_atomic(&self) -> Result<()>;
}

/// A trait representing map-like storage operations with read-only capabilities.
pub trait MapRead<
    'a,
    K: 'a + Copy + Clone + PartialEq + Eq + Hash + Serialize + Deserialize<'a> + Sync,
    V: 'a + Clone + PartialEq + Eq + Serialize + Deserialize<'a> + Sync,
>
{
    type PendingIterator: Iterator<Item = (Cow<'a, K>, Option<Cow<'a, V>>)>;
    type Iterator: Iterator<Item = (Cow<'a, K>, Cow<'a, V>)>;
    type Keys: Iterator<Item = Cow<'a, K>>;
    type Values: Iterator<Item = Cow<'a, V>>;

    ///
    /// Returns `true` if the given key exists in the map.
    ///
    fn contains_key_confirmed<Q>(&self, key: &Q) -> Result<bool>
    where
        K: Borrow<Q>,
        Q: PartialEq + Eq + Hash + Serialize + ?Sized;

    ///
    /// Returns `true` if the given key exists in the map.
    /// This method first checks the atomic batch, and if it does not exist, then checks the map.
    ///
    fn contains_key_speculative<Q>(&self, key: &Q) -> Result<bool>
    where
        K: Borrow<Q>,
        Q: PartialEq + Eq + Hash + Serialize + ?Sized;

    ///
    /// Returns the value for the given key from the map, if it exists.
    ///
    fn get_confirmed<Q>(&'a self, key: &Q) -> Result<Option<Cow<'a, V>>>
    where
        K: Borrow<Q>,
        Q: PartialEq + Eq + Hash + Serialize + ?Sized;

    ///
    /// Returns the current value for the given key if it is scheduled
    /// to be inserted as part of an atomic batch.
    ///
    /// If the key does not exist, returns `None`.
    /// If the key is removed in the batch, returns `Some(None)`.
    /// If the key is inserted in the batch, returns `Some(Some(value))`.
    ///
    fn get_pending<Q>(&self, key: &Q) -> Option<Option<V>>
    where
        K: Borrow<Q>,
        Q: PartialEq + Eq + Hash + Serialize + ?Sized;

    ///
    /// Returns the value for the given key from the atomic batch first, if it exists,
    /// or return from the map, otherwise.
    ///
    fn get_speculative<Q>(&'a self, key: &Q) -> Result<Option<Cow<'a, V>>>
    where
        K: Borrow<Q>,
        Q: PartialEq + Eq + Hash + Serialize + ?Sized,
    {
        // Return early in case of errors in order to not conceal them.
        let map_value = self.get_confirmed(key)?;

        // Retrieve the atomic batch value, if it exists.
        let atomic_batch_value = self.get_pending(key);

        // Return the atomic batch value, if it exists, or the map value, otherwise.
        match atomic_batch_value {
            Some(Some(value)) => Ok(Some(Cow::Owned(value))),
            Some(None) => Ok(None),
            None => Ok(map_value),
        }
    }

    ///
    /// Returns an iterator visiting each key-value pair in the atomic batch.
    ///
    fn iter_pending(&'a self) -> Self::PendingIterator;

    ///
    /// Returns an iterator visiting each key-value pair in the map.
    ///
    fn iter_confirmed(&'a self) -> Self::Iterator;

    ///
    /// Returns an iterator over each key in the map.
    ///
    fn keys_confirmed(&'a self) -> Self::Keys;

    ///
    /// Returns an iterator over each value in the map.
    ///
    fn values_confirmed(&'a self) -> Self::Values;
}

/// This macro executes the given block of operations as a new atomic write batch IFF there is no
/// atomic write batch in progress yet. This ensures that complex atomic operations consisting of
/// multiple lower-level operations - which might also need to be atomic if executed individually -
/// are executed as a single large atomic operation regardless.
#[macro_export]
macro_rules! atomic_batch_scope {
    ($self:expr, $ops:block) => {{
        // Check if an atomic batch write is already in progress. If there isn't one, this means
        // this operation is a "top-level" one and is the one to start and finalize the batch.
        let is_atomic_in_progress = $self.is_atomic_in_progress();

        // Start an atomic batch write operation IFF it's not already part of one.
        match is_atomic_in_progress {
            true => $self.atomic_checkpoint(),
            false => $self.start_atomic(),
        }

        // Wrap the operations that should be batched in a closure to be able to rewind the batch on error.
        let run_atomic_ops = || -> Result<_> { $ops };

        // Run the atomic operations.
        match run_atomic_ops() {
            // Save this atomic batch scope and return.
            Ok(result) => match is_atomic_in_progress {
                // A 'true' implies this is a nested atomic batch scope.
                true => Ok(result),
                // A 'false' implies this is the top-level calling scope.
                // Commit the atomic batch IFF it's the top-level calling scope.
                false => $self.finish_atomic().map(|_| result),
            },
            // Rewind this atomic batch scope.
            Err(err) => {
                $self.atomic_rewind();
                Err(err)
            }
        }
    }};
}

/// A top-level helper macro to perform the finalize operation on a list of transactions.
#[macro_export]
macro_rules! atomic_finalize {
    ($self:expr, $ops:block) => {{
        // Ensure that there is no atomic batch write in progress.
        if $self.is_atomic_in_progress() {
            // We intentionally 'bail!' here instead of passing an Err() to the caller because
            // this is a top-level operation and the caller must fix the issue.
            bail!("Cannot start an atomic batch write operation while another one is already in progress.")
        }

        // Start the atomic batch.
        $self.start_atomic();

        // Wrap the operations that should be batched in a closure to be able to abort the entire
        // write batch if any of them fails.
        let run_atomic_ops = || -> Result<()> { $ops };

        // Run the atomic operations.
        match run_atomic_ops() {
            // Finalize the batch if all operations have succeeded.
            Ok(result) => {
                $self.finish_atomic()?;
                Ok(result)
            }
            // Abort the batch if any of the associated operations has failed.
            Err(err) => {
                $self.abort_atomic();
                Err(err)
            }
        }
    }};
}
