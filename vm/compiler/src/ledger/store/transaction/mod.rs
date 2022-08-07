// Copyright (C) 2019-2022 Aleo Systems Inc.
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

mod deployment;
pub use deployment::*;

mod execution;
pub use execution::*;

use crate::{
    cow_to_copied,
    ledger::{
        map::{memory_map::MemoryMap, Map, MapRead},
        AdditionalFee,
        Transaction,
    },
    process::{Deployment, Execution},
};
use console::{network::prelude::*, program::ProgramID};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionType {
    /// A transaction that is a deployment.
    Deploy,
    /// A transaction that is an execution.
    Execute,
}

/// A trait for transaction storage.
pub trait TransactionStorage<N: Network>: Clone {
    /// The mapping of `transaction ID` to `transaction type`.
    type IDMap: for<'a> Map<'a, N::TransactionID, TransactionType>;
    /// The deployment storage.
    type DeploymentStorage: DeploymentStorage<N>;
    /// The execution storage.
    type ExecutionStorage: ExecutionStorage<N>;

    /// Returns the ID map.
    fn id_map(&self) -> &Self::IDMap;
    /// Returns the deployment store.
    fn deployment_store(&self) -> &DeploymentStore<N, Self::DeploymentStorage>;
    /// Returns the execution store.
    fn execution_store(&self) -> &ExecutionStore<N, Self::ExecutionStorage>;

    /// Returns the transaction ID that contains the given `transition ID`.
    fn find_transaction_id(&self, transition_id: &N::TransitionID) -> Result<Option<N::TransactionID>> {
        self.execution_store().find_transaction_id(transition_id)
    }

    /// Returns the transaction ID that contains the given `program ID`.
    fn find_deployment_id(&self, program_id: &ProgramID<N>) -> Result<Option<N::TransactionID>> {
        self.deployment_store().find_transaction_id(program_id)
    }

    /// Returns the transaction for the given `transaction ID`.
    fn get_transaction(&self, transaction_id: &N::TransactionID) -> Result<Option<Transaction<N>>> {
        // Retrieve the transaction type.
        let transaction_type = match self.id_map().get(transaction_id)? {
            Some(transaction_type) => cow_to_copied!(transaction_type),
            None => bail!("Failed to get the type for transaction '{transaction_id}'"),
        };
        // Retrieve the transaction.
        match transaction_type {
            // Return the deployment transaction.
            TransactionType::Deploy => self.deployment_store().get_transaction(transaction_id),
            // Return the execution transaction.
            TransactionType::Execute => self.execution_store().get_transaction(transaction_id),
        }
    }

    /// Stores the given `transaction` into storage.
    fn insert(&self, transaction: &Transaction<N>) -> Result<()> {
        match transaction {
            Transaction::Deploy(..) => {
                // Store the transaction type.
                self.id_map().insert(transaction.id(), TransactionType::Deploy)?;
                // Store the deployment transaction.
                self.deployment_store().insert(transaction)
            }
            Transaction::Execute(..) => {
                // Store the transaction type.
                self.id_map().insert(transaction.id(), TransactionType::Execute)?;
                // Store the execution transaction.
                self.execution_store().insert(transaction)
            }
        }
    }

    /// Removes the transaction for the given `transaction ID`.
    fn remove(&self, transaction_id: &N::TransactionID) -> Result<()> {
        // Retrieve the transaction type.
        let transaction_type = match self.id_map().get(transaction_id)? {
            Some(transaction_type) => cow_to_copied!(transaction_type),
            None => bail!("Failed to get the type for transaction '{transaction_id}'"),
        };

        // Remove the transaction type.
        self.id_map().remove(transaction_id)?;
        // Remove the transaction.
        match transaction_type {
            // Remove the deployment transaction.
            TransactionType::Deploy => self.deployment_store().remove(transaction_id),
            // Remove the execution transaction.
            TransactionType::Execute => self.execution_store().remove(transaction_id),
        }
    }
}

/// An in-memory transaction storage.
#[derive(Clone)]
pub struct TransactionMemory<N: Network> {
    /// The mapping of `transaction ID` to `transaction type`.
    id_map: MemoryMap<N::TransactionID, TransactionType>,
    /// The deployment store.
    deployment_store: DeploymentStore<N, DeploymentMemory<N>>,
    /// The execution store.
    execution_store: ExecutionStore<N, ExecutionMemory<N>>,
}

impl<N: Network> TransactionMemory<N> {
    /// Creates a new in-memory transaction storage.
    pub fn new(
        deployment_store: DeploymentStore<N, DeploymentMemory<N>>,
        execution_store: ExecutionStore<N, ExecutionMemory<N>>,
    ) -> Self {
        Self { id_map: MemoryMap::default(), deployment_store, execution_store }
    }
}

#[rustfmt::skip]
impl<N: Network> TransactionStorage<N> for TransactionMemory<N> {
    type IDMap = MemoryMap<N::TransactionID, TransactionType>;
    type DeploymentStorage = DeploymentMemory<N>;
    type ExecutionStorage = ExecutionMemory<N>;

    /// Returns the ID map.
    fn id_map(&self) -> &Self::IDMap {
        &self.id_map
    }

    /// Returns the deployment store.
    fn deployment_store(&self) -> &DeploymentStore<N, Self::DeploymentStorage> {
        &self.deployment_store
    }

    /// Returns the execution store.
    fn execution_store(&self) -> &ExecutionStore<N, Self::ExecutionStorage> {
        &self.execution_store
    }
}

/// The transaction store.
#[derive(Clone)]
pub struct TransactionStore<N: Network, T: TransactionStorage<N>> {
    /// The map of `transaction ID` to `transaction type`.
    transaction_ids: T::IDMap,
    /// The transaction storage.
    storage: T,
}

impl<N: Network, T: TransactionStorage<N>> TransactionStore<N, T> {
    /// Initializes a new execution store.
    pub fn new(storage: T) -> Self {
        Self { transaction_ids: storage.id_map().clone(), storage }
    }

    /// Stores the given `transaction` into storage.
    pub fn insert(&self, transaction: &Transaction<N>) -> Result<()> {
        self.storage.insert(transaction)
    }

    /// Removes the transaction for the given `transaction ID`.
    pub fn remove(&self, transaction_id: &N::TransactionID) -> Result<()> {
        self.storage.remove(transaction_id)
    }
}

impl<N: Network, T: TransactionStorage<N>> TransactionStore<N, T> {
    /// Returns the transaction for the given `transaction ID`.
    pub fn get_transaction(&self, transaction_id: &N::TransactionID) -> Result<Option<Transaction<N>>> {
        self.storage.get_transaction(transaction_id)
    }

    /// Returns the deployment for the given `transaction ID`.
    pub fn get_deployment(&self, transaction_id: &N::TransactionID) -> Result<Option<Deployment<N>>> {
        // Retrieve the transaction type.
        let transaction_type = match self.transaction_ids.get(transaction_id)? {
            Some(transaction_type) => cow_to_copied!(transaction_type),
            None => bail!("Failed to get the type for transaction '{transaction_id}'"),
        };
        // Retrieve the deployment.
        match transaction_type {
            // Return the deployment.
            TransactionType::Deploy => self.storage.deployment_store().get_deployment(transaction_id),
            // Throw an error.
            TransactionType::Execute => bail!("Tried to get a deployment for execution transaction '{transaction_id}'"),
        }
    }

    /// Returns the execution for the given `transaction ID`.
    pub fn get_execution(&self, transaction_id: &N::TransactionID) -> Result<Option<Execution<N>>> {
        // Retrieve the transaction type.
        let transaction_type = match self.transaction_ids.get(transaction_id)? {
            Some(transaction_type) => cow_to_copied!(transaction_type),
            None => bail!("Failed to get the type for transaction '{transaction_id}'"),
        };
        // Retrieve the execution.
        match transaction_type {
            // Throw an error.
            TransactionType::Deploy => bail!("Tried to get an execution for deployment transaction '{transaction_id}'"),
            // Return the execution.
            TransactionType::Execute => self.storage.execution_store().get_execution(transaction_id),
        }
    }

    /// Returns the edition for the given `transaction ID`.
    pub fn get_edition(&self, transaction_id: &N::TransactionID) -> Result<Option<u16>> {
        // Retrieve the transaction type.
        let transaction_type = match self.transaction_ids.get(transaction_id)? {
            Some(transaction_type) => cow_to_copied!(transaction_type),
            None => bail!("Failed to get the type for transaction '{transaction_id}'"),
        };
        // Retrieve the edition.
        match transaction_type {
            TransactionType::Deploy => {
                // Retrieve the program ID.
                let program_id = self.storage.deployment_store().get_program_id(transaction_id)?;
                // Return the edition.
                match program_id {
                    Some(program_id) => self.storage.deployment_store().get_edition(&program_id),
                    None => bail!("Failed to get the program ID for deployment transaction '{transaction_id}'"),
                }
            }
            // Return the edition.
            TransactionType::Execute => self.storage.execution_store().get_edition(transaction_id),
        }
    }

    /// Returns the additional fee for the given `transaction ID`.
    pub fn get_additional_fee(&self, transaction_id: &N::TransactionID) -> Result<Option<AdditionalFee<N>>> {
        // Retrieve the transaction type.
        let transaction_type = match self.transaction_ids.get(transaction_id)? {
            Some(transaction_type) => cow_to_copied!(transaction_type),
            None => bail!("Failed to get the type for transaction '{transaction_id}'"),
        };
        // Retrieve the fee.
        match transaction_type {
            // Return the fee.
            TransactionType::Deploy => self.storage.deployment_store().get_additional_fee(transaction_id),
            // Return the fee.
            TransactionType::Execute => self.storage.execution_store().get_additional_fee(transaction_id),
        }
    }
}

impl<N: Network, T: TransactionStorage<N>> TransactionStore<N, T> {
    /// Returns an iterator over the transaction IDs, for all transitions in `self`.
    pub fn transaction_ids(&self) -> impl '_ + Iterator<Item = Cow<'_, N::TransactionID>> {
        self.transaction_ids.keys()
    }
}

impl<N: Network, T: TransactionStorage<N>> TransactionStore<N, T> {
    /// Returns `true` if the given transaction ID exists.
    pub fn contains_transaction_id(&self, transaction_id: &N::TransactionID) -> Result<bool> {
        self.transaction_ids.contains_key(transaction_id)
    }
}
