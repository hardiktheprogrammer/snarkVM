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

use crate::{
    atomic_batch_scope,
    block::Transaction,
    cow_to_cloned,
    cow_to_copied,
    process::{Deployment, Fee},
    program::Program,
    snark::{Certificate, Proof, VerifyingKey},
    store::{
        helpers::{Map, MapRead},
        TransitionStorage,
        TransitionStore,
    },
};
use console::{
    network::prelude::*,
    program::{Identifier, ProgramID, ProgramOwner},
};

use anyhow::Result;
use core::marker::PhantomData;
use std::borrow::Cow;

/// A trait for deployment storage.
pub trait DeploymentStorage<N: Network>: Clone + Send + Sync {
    /// The mapping of `transaction ID` to `program ID`.
    type IDMap: for<'a> Map<'a, N::TransactionID, ProgramID<N>>;
    /// The mapping of `program ID` to `edition`.
    type EditionMap: for<'a> Map<'a, ProgramID<N>, u16>;
    /// The mapping of `(program ID, edition)` to `transaction ID`.
    type ReverseIDMap: for<'a> Map<'a, (ProgramID<N>, u16), N::TransactionID>;
    /// The mapping of `(program ID, edition)` to `ProgramOwner`.
    type OwnerMap: for<'a> Map<'a, (ProgramID<N>, u16), ProgramOwner<N>>;
    /// The mapping of `(program ID, edition)` to `program`.
    type ProgramMap: for<'a> Map<'a, (ProgramID<N>, u16), Program<N>>;
    /// The mapping of `(program ID, function name, edition)` to `verifying key`.
    type VerifyingKeyMap: for<'a> Map<'a, (ProgramID<N>, Identifier<N>, u16), VerifyingKey<N>>;
    /// The mapping of `(program ID, function name, edition)` to `certificate`.
    type CertificateMap: for<'a> Map<'a, (ProgramID<N>, Identifier<N>, u16), Certificate<N>>;
    /// The mapping of `transaction ID` to `(fee transition ID, global state root, inclusion proof)`.
    type FeeMap: for<'a> Map<'a, N::TransactionID, (N::TransitionID, N::StateRoot, Option<Proof<N>>)>;
    /// The mapping of `fee transition ID` to `transaction ID`.
    type ReverseFeeMap: for<'a> Map<'a, N::TransitionID, N::TransactionID>;

    /// The transition storage.
    type TransitionStorage: TransitionStorage<N>;

    /// Initializes the deployment storage.
    fn open(transition_store: TransitionStore<N, Self::TransitionStorage>) -> Result<Self>;

    /// Returns the ID map.
    fn id_map(&self) -> &Self::IDMap;
    /// Returns the edition map.
    fn edition_map(&self) -> &Self::EditionMap;
    /// Returns the reverse ID map.
    fn reverse_id_map(&self) -> &Self::ReverseIDMap;
    /// Returns the owner map.
    fn owner_map(&self) -> &Self::OwnerMap;
    /// Returns the program map.
    fn program_map(&self) -> &Self::ProgramMap;
    /// Returns the verifying key map.
    fn verifying_key_map(&self) -> &Self::VerifyingKeyMap;
    /// Returns the certificate map.
    fn certificate_map(&self) -> &Self::CertificateMap;
    /// Returns the fee map.
    fn fee_map(&self) -> &Self::FeeMap;
    /// Returns the reverse fee map.
    fn reverse_fee_map(&self) -> &Self::ReverseFeeMap;
    /// Returns the transition storage.
    fn transition_store(&self) -> &TransitionStore<N, Self::TransitionStorage>;

    /// Returns the optional development ID.
    fn dev(&self) -> Option<u16> {
        self.transition_store().dev()
    }

    /// Starts an atomic batch write operation.
    fn start_atomic(&self) {
        self.id_map().start_atomic();
        self.edition_map().start_atomic();
        self.reverse_id_map().start_atomic();
        self.owner_map().start_atomic();
        self.program_map().start_atomic();
        self.verifying_key_map().start_atomic();
        self.certificate_map().start_atomic();
        self.fee_map().start_atomic();
        self.reverse_fee_map().start_atomic();
        self.transition_store().start_atomic();
    }

    /// Checks if an atomic batch is in progress.
    fn is_atomic_in_progress(&self) -> bool {
        self.id_map().is_atomic_in_progress()
            || self.edition_map().is_atomic_in_progress()
            || self.reverse_id_map().is_atomic_in_progress()
            || self.owner_map().is_atomic_in_progress()
            || self.program_map().is_atomic_in_progress()
            || self.verifying_key_map().is_atomic_in_progress()
            || self.certificate_map().is_atomic_in_progress()
            || self.fee_map().is_atomic_in_progress()
            || self.reverse_fee_map().is_atomic_in_progress()
            || self.transition_store().is_atomic_in_progress()
    }

    /// Checkpoints the atomic batch.
    fn atomic_checkpoint(&self) {
        self.id_map().atomic_checkpoint();
        self.edition_map().atomic_checkpoint();
        self.reverse_id_map().atomic_checkpoint();
        self.owner_map().atomic_checkpoint();
        self.program_map().atomic_checkpoint();
        self.verifying_key_map().atomic_checkpoint();
        self.certificate_map().atomic_checkpoint();
        self.fee_map().atomic_checkpoint();
        self.reverse_fee_map().atomic_checkpoint();
        self.transition_store().atomic_checkpoint();
    }

    /// Rewinds the atomic batch to the previous checkpoint.
    fn atomic_rewind(&self) {
        self.id_map().atomic_rewind();
        self.edition_map().atomic_rewind();
        self.reverse_id_map().atomic_rewind();
        self.owner_map().atomic_rewind();
        self.program_map().atomic_rewind();
        self.verifying_key_map().atomic_rewind();
        self.certificate_map().atomic_rewind();
        self.fee_map().atomic_rewind();
        self.reverse_fee_map().atomic_rewind();
        self.transition_store().atomic_rewind();
    }

    /// Aborts an atomic batch write operation.
    fn abort_atomic(&self) {
        self.id_map().abort_atomic();
        self.edition_map().abort_atomic();
        self.reverse_id_map().abort_atomic();
        self.owner_map().abort_atomic();
        self.program_map().abort_atomic();
        self.verifying_key_map().abort_atomic();
        self.certificate_map().abort_atomic();
        self.fee_map().abort_atomic();
        self.reverse_fee_map().abort_atomic();
        self.transition_store().abort_atomic();
    }

    /// Finishes an atomic batch write operation.
    fn finish_atomic(&self) -> Result<()> {
        self.id_map().finish_atomic()?;
        self.edition_map().finish_atomic()?;
        self.reverse_id_map().finish_atomic()?;
        self.owner_map().finish_atomic()?;
        self.program_map().finish_atomic()?;
        self.verifying_key_map().finish_atomic()?;
        self.certificate_map().finish_atomic()?;
        self.fee_map().finish_atomic()?;
        self.reverse_fee_map().finish_atomic()?;
        self.transition_store().finish_atomic()
    }

    /// Stores the given `deployment transaction` pair into storage.
    fn insert(&self, transaction: &Transaction<N>) -> Result<()> {
        // Ensure the transaction is a deployment.
        let (transaction_id, owner, deployment, fee) = match transaction {
            Transaction::Deploy(transaction_id, owner, deployment, fee) => (transaction_id, owner, deployment, fee),
            Transaction::Execute(..) => {
                bail!("Attempted to insert non-deployment transaction into deployment storage.")
            }
        };

        // Ensure the deployment is ordered.
        if let Err(error) = deployment.check_is_ordered() {
            bail!("Failed to insert malformed deployment transaction: {error}")
        }

        // Retrieve the edition.
        let edition = deployment.edition();
        // Retrieve the program.
        let program = deployment.program();
        // Retrieve the program ID.
        let program_id = *program.id();

        atomic_batch_scope!(self, {
            // Store the program ID.
            self.id_map().insert(*transaction_id, program_id)?;
            // Store the edition.
            self.edition_map().insert(program_id, edition)?;

            // Store the reverse program ID.
            self.reverse_id_map().insert((program_id, edition), *transaction_id)?;
            // Store the owner.
            self.owner_map().insert((program_id, edition), *owner)?;
            // Store the program.
            self.program_map().insert((program_id, edition), program.clone())?;

            // Store the verifying keys and certificates.
            for (function_name, (verifying_key, certificate)) in deployment.verifying_keys() {
                // Store the verifying key.
                self.verifying_key_map().insert((program_id, *function_name, edition), verifying_key.clone())?;
                // Store the certificate.
                self.certificate_map().insert((program_id, *function_name, edition), certificate.clone())?;
            }

            // Store the fee.
            self.fee_map().insert(
                *transaction_id,
                (*fee.transition_id(), fee.global_state_root(), fee.inclusion_proof().cloned()),
            )?;
            self.reverse_fee_map().insert(*fee.transition_id(), *transaction_id)?;

            // Store the fee transition.
            self.transition_store().insert(fee)?;

            Ok(())
        })
    }

    /// Removes the deployment transaction for the given `transaction ID`.
    fn remove(&self, transaction_id: &N::TransactionID) -> Result<()> {
        // Retrieve the program ID.
        let program_id = match self.get_program_id(transaction_id)? {
            Some(edition) => edition,
            None => bail!("Failed to get the program ID for transaction '{transaction_id}'"),
        };
        // Retrieve the edition.
        let edition = match self.get_edition(&program_id)? {
            Some(edition) => edition,
            None => bail!("Failed to locate the edition for program '{program_id}'"),
        };
        // Retrieve the program.
        let program = match self.program_map().get_confirmed(&(program_id, edition))? {
            Some(program) => cow_to_cloned!(program),
            None => bail!("Failed to locate program '{program_id}' for transaction '{transaction_id}'"),
        };
        // Retrieve the fee transition ID.
        let (transition_id, _, _) = match self.fee_map().get_confirmed(transaction_id)? {
            Some(fee_id) => cow_to_cloned!(fee_id),
            None => bail!("Failed to locate the fee transition ID for transaction '{transaction_id}'"),
        };

        atomic_batch_scope!(self, {
            // Remove the program ID.
            self.id_map().remove(transaction_id)?;
            // Remove the edition.
            self.edition_map().remove(&program_id)?;

            // Remove the reverse program ID.
            self.reverse_id_map().remove(&(program_id, edition))?;
            // Remove the owner.
            self.owner_map().remove(&(program_id, edition))?;
            // Remove the program.
            self.program_map().remove(&(program_id, edition))?;

            // Remove the verifying keys and certificates.
            for function_name in program.functions().keys() {
                // Remove the verifying key.
                self.verifying_key_map().remove(&(program_id, *function_name, edition))?;
                // Remove the certificate.
                self.certificate_map().remove(&(program_id, *function_name, edition))?;
            }

            // Remove the fee.
            self.fee_map().remove(transaction_id)?;
            self.reverse_fee_map().remove(&transition_id)?;

            // Remove the fee transition.
            self.transition_store().remove(&transition_id)?;

            Ok(())
        })
    }

    /// Returns the transaction ID that contains the given `program ID`.
    fn find_transaction_id_from_program_id(&self, program_id: &ProgramID<N>) -> Result<Option<N::TransactionID>> {
        // Retrieve the edition.
        let edition = match self.get_edition(program_id)? {
            Some(edition) => edition,
            None => return Ok(None),
        };
        // Retrieve the transaction ID.
        match self.reverse_id_map().get_confirmed(&(*program_id, edition))? {
            Some(transaction_id) => Ok(Some(cow_to_copied!(transaction_id))),
            None => bail!("Failed to find the transaction ID for program '{program_id}' (edition {edition})"),
        }
    }

    /// Returns the transaction ID that contains the given `transition ID`.
    fn find_transaction_id_from_transition_id(
        &self,
        transition_id: &N::TransitionID,
    ) -> Result<Option<N::TransactionID>> {
        match self.reverse_fee_map().get_confirmed(transition_id)? {
            Some(transaction_id) => Ok(Some(cow_to_copied!(transaction_id))),
            None => Ok(None),
        }
    }

    /// Returns the program ID for the given `transaction ID`.
    fn get_program_id(&self, transaction_id: &N::TransactionID) -> Result<Option<ProgramID<N>>> {
        // Retrieve the program ID.
        match self.id_map().get_confirmed(transaction_id)? {
            Some(program_id) => Ok(Some(cow_to_copied!(program_id))),
            None => Ok(None),
        }
    }

    /// Returns the edition for the given `program ID`.
    fn get_edition(&self, program_id: &ProgramID<N>) -> Result<Option<u16>> {
        match self.edition_map().get_confirmed(program_id)? {
            Some(edition) => Ok(Some(cow_to_copied!(edition))),
            None => Ok(None),
        }
    }

    /// Returns the program for the given `program ID`.
    fn get_program(&self, program_id: &ProgramID<N>) -> Result<Option<Program<N>>> {
        // Retrieve the edition.
        let edition = match self.get_edition(program_id)? {
            Some(edition) => edition,
            None => return Ok(None),
        };
        // Retrieve the program.
        match self.program_map().get_confirmed(&(*program_id, edition))? {
            Some(program) => Ok(Some(cow_to_cloned!(program))),
            None => bail!("Failed to get program '{program_id}' (edition {edition})"),
        }
    }

    /// Returns the verifying key for the given `program ID` and `function name`.
    fn get_verifying_key(
        &self,
        program_id: &ProgramID<N>,
        function_name: &Identifier<N>,
    ) -> Result<Option<VerifyingKey<N>>> {
        // Retrieve the edition.
        let edition = match self.get_edition(program_id)? {
            Some(edition) => edition,
            None => return Ok(None),
        };
        // Retrieve the verifying key.
        match self.verifying_key_map().get_confirmed(&(*program_id, *function_name, edition))? {
            Some(verifying_key) => Ok(Some(cow_to_cloned!(verifying_key))),
            None => bail!("Failed to get the verifying key for '{program_id}/{function_name}' (edition {edition})"),
        }
    }

    /// Returns the certificate for the given `program ID` and `function name`.
    fn get_certificate(
        &self,
        program_id: &ProgramID<N>,
        function_name: &Identifier<N>,
    ) -> Result<Option<Certificate<N>>> {
        // Retrieve the edition.
        let edition = match self.get_edition(program_id)? {
            Some(edition) => edition,
            None => return Ok(None),
        };
        // Retrieve the certificate.
        match self.certificate_map().get_confirmed(&(*program_id, *function_name, edition))? {
            Some(certificate) => Ok(Some(cow_to_cloned!(certificate))),
            None => bail!("Failed to get the certificate for '{program_id}/{function_name}' (edition {edition})"),
        }
    }

    /// Returns the deployment for the given `transaction ID`.
    fn get_deployment(&self, transaction_id: &N::TransactionID) -> Result<Option<Deployment<N>>> {
        // Retrieve the program ID.
        let program_id = match self.get_program_id(transaction_id)? {
            Some(edition) => edition,
            None => return Ok(None),
        };
        // Retrieve the edition.
        let edition = match self.get_edition(&program_id)? {
            Some(edition) => edition,
            None => bail!("Failed to get the edition for program '{program_id}'"),
        };
        // Retrieve the program.
        let program = match self.program_map().get_confirmed(&(program_id, edition))? {
            Some(program) => cow_to_cloned!(program),
            None => bail!("Failed to get the deployed program '{program_id}' (edition {edition})"),
        };

        // Initialize a vector for the verifying keys and certificates.
        let mut verifying_keys = Vec::with_capacity(program.functions().len());

        // Retrieve the verifying keys and certificates.
        for function_name in program.functions().keys() {
            // Retrieve the verifying key.
            let verifying_key = match self.verifying_key_map().get_confirmed(&(program_id, *function_name, edition))? {
                Some(verifying_key) => cow_to_cloned!(verifying_key),
                None => bail!("Failed to get the verifying key for '{program_id}/{function_name}' (edition {edition})"),
            };
            // Retrieve the certificate.
            let certificate = match self.certificate_map().get_confirmed(&(program_id, *function_name, edition))? {
                Some(certificate) => cow_to_cloned!(certificate),
                None => bail!("Failed to get the certificate for '{program_id}/{function_name}' (edition {edition})"),
            };
            // Add the verifying key and certificate to the deployment.
            verifying_keys.push((*function_name, (verifying_key, certificate)));
        }

        // Return the deployment.
        Ok(Some(Deployment::new(edition, program, verifying_keys)?))
    }

    /// Returns the fee for the given `transaction ID`.
    fn get_fee(&self, transaction_id: &N::TransactionID) -> Result<Option<Fee<N>>> {
        // Retrieve the fee transition ID.
        let (fee_transition_id, global_state_root, inclusion_proof) =
            match self.fee_map().get_confirmed(transaction_id)? {
                Some(fee) => cow_to_cloned!(fee),
                None => return Ok(None),
            };
        // Retrieve the fee transition.
        match self.transition_store().get_transition(&fee_transition_id)? {
            Some(transition) => Ok(Some(Fee::from(transition, global_state_root, inclusion_proof))),
            None => bail!("Failed to locate the fee transition for transaction '{transaction_id}'"),
        }
    }

    /// Returns the owner for the given `program ID`.
    fn get_owner(&self, program_id: &ProgramID<N>) -> Result<Option<ProgramOwner<N>>> {
        // TODO (raychu86): Consider program upgrades and edition changes.
        // Retrieve the edition.
        let edition = match self.get_edition(program_id)? {
            Some(edition) => edition,
            None => return Ok(None),
        };

        // Retrieve the owner.
        match self.owner_map().get_confirmed(&(*program_id, edition))? {
            Some(owner) => Ok(Some(cow_to_copied!(owner))),
            None => bail!("Failed to find the Owner for program '{program_id}' (edition {edition})"),
        }
    }

    /// Returns the transaction for the given `transaction ID`.
    fn get_transaction(&self, transaction_id: &N::TransactionID) -> Result<Option<Transaction<N>>> {
        // Retrieve the deployment.
        let deployment = match self.get_deployment(transaction_id)? {
            Some(deployment) => deployment,
            None => return Ok(None),
        };
        // Retrieve the fee.
        let fee = match self.get_fee(transaction_id)? {
            Some(fee) => fee,
            None => bail!("Failed to get the fee for transaction '{transaction_id}'"),
        };

        // Retrieve the owner.
        let owner = match self.get_owner(deployment.program_id())? {
            Some(owner) => owner,
            None => bail!("Failed to get the owner for transaction '{transaction_id}'"),
        };

        // Construct the deployment transaction.
        let deployment_transaction = Transaction::from_deployment(owner, deployment, fee)?;
        // Ensure the transaction ID matches.
        match *transaction_id == deployment_transaction.id() {
            true => Ok(Some(deployment_transaction)),
            false => bail!("The deployment transaction ID does not match '{transaction_id}'"),
        }
    }
}

/// The deployment store.
#[derive(Clone)]
pub struct DeploymentStore<N: Network, D: DeploymentStorage<N>> {
    /// The deployment storage.
    storage: D,
    /// PhantomData.
    _phantom: PhantomData<N>,
}

impl<N: Network, D: DeploymentStorage<N>> DeploymentStore<N, D> {
    /// Initializes the deployment store.
    pub fn open(transition_store: TransitionStore<N, D::TransitionStorage>) -> Result<Self> {
        // Initialize the deployment storage.
        let storage = D::open(transition_store)?;
        // Return the deployment store.
        Ok(Self { storage, _phantom: PhantomData })
    }

    /// Initializes a deployment store from storage.
    pub fn from(storage: D) -> Self {
        Self { storage, _phantom: PhantomData }
    }

    /// Stores the given `deployment transaction` into storage.
    pub fn insert(&self, transaction: &Transaction<N>) -> Result<()> {
        self.storage.insert(transaction)
    }

    /// Removes the transaction for the given `transaction ID`.
    pub fn remove(&self, transaction_id: &N::TransactionID) -> Result<()> {
        self.storage.remove(transaction_id)
    }

    /// Starts an atomic batch write operation.
    pub fn start_atomic(&self) {
        self.storage.start_atomic();
    }

    /// Checks if an atomic batch is in progress.
    pub fn is_atomic_in_progress(&self) -> bool {
        self.storage.is_atomic_in_progress()
    }

    /// Checkpoints the atomic batch.
    pub fn atomic_checkpoint(&self) {
        self.storage.atomic_checkpoint();
    }

    /// Rewinds the atomic batch to the previous checkpoint.
    pub fn atomic_rewind(&self) {
        self.storage.atomic_rewind();
    }

    /// Aborts an atomic batch write operation.
    pub fn abort_atomic(&self) {
        self.storage.abort_atomic();
    }

    /// Finishes an atomic batch write operation.
    pub fn finish_atomic(&self) -> Result<()> {
        self.storage.finish_atomic()
    }

    /// Returns the optional development ID.
    pub fn dev(&self) -> Option<u16> {
        self.storage.dev()
    }
}

impl<N: Network, D: DeploymentStorage<N>> DeploymentStore<N, D> {
    /// Returns the transaction for the given `transaction ID`.
    pub fn get_transaction(&self, transaction_id: &N::TransactionID) -> Result<Option<Transaction<N>>> {
        self.storage.get_transaction(transaction_id)
    }

    /// Returns the deployment for the given `transaction ID`.
    pub fn get_deployment(&self, transaction_id: &N::TransactionID) -> Result<Option<Deployment<N>>> {
        self.storage.get_deployment(transaction_id)
    }

    /// Returns the edition for the given `program ID`.
    pub fn get_edition(&self, program_id: &ProgramID<N>) -> Result<Option<u16>> {
        self.storage.get_edition(program_id)
    }

    /// Returns the program ID for the given `transaction ID`.
    pub fn get_program_id(&self, transaction_id: &N::TransactionID) -> Result<Option<ProgramID<N>>> {
        self.storage.get_program_id(transaction_id)
    }

    /// Returns the program for the given `program ID`.
    pub fn get_program(&self, program_id: &ProgramID<N>) -> Result<Option<Program<N>>> {
        self.storage.get_program(program_id)
    }

    /// Returns the verifying key for the given `(program ID, function name)`.
    pub fn get_verifying_key(
        &self,
        program_id: &ProgramID<N>,
        function_name: &Identifier<N>,
    ) -> Result<Option<VerifyingKey<N>>> {
        self.storage.get_verifying_key(program_id, function_name)
    }

    /// Returns the certificate for the given `(program ID, function name)`.
    pub fn get_certificate(
        &self,
        program_id: &ProgramID<N>,
        function_name: &Identifier<N>,
    ) -> Result<Option<Certificate<N>>> {
        self.storage.get_certificate(program_id, function_name)
    }

    /// Returns the fee for the given `transaction ID`.
    pub fn get_fee(&self, transaction_id: &N::TransactionID) -> Result<Option<Fee<N>>> {
        self.storage.get_fee(transaction_id)
    }
}

impl<N: Network, D: DeploymentStorage<N>> DeploymentStore<N, D> {
    /// Returns the transaction ID that deployed the given `program ID`.
    pub fn find_transaction_id_from_program_id(&self, program_id: &ProgramID<N>) -> Result<Option<N::TransactionID>> {
        self.storage.find_transaction_id_from_program_id(program_id)
    }

    /// Returns the transaction ID that deployed the given `transition ID`.
    pub fn find_transaction_id_from_transition_id(
        &self,
        transition_id: &N::TransitionID,
    ) -> Result<Option<N::TransactionID>> {
        self.storage.find_transaction_id_from_transition_id(transition_id)
    }
}

impl<N: Network, D: DeploymentStorage<N>> DeploymentStore<N, D> {
    /// Returns `true` if the given program ID exists.
    pub fn contains_program_id(&self, program_id: &ProgramID<N>) -> Result<bool> {
        self.storage.edition_map().contains_key_confirmed(program_id)
    }
}

impl<N: Network, D: DeploymentStorage<N>> DeploymentStore<N, D> {
    /// Returns an iterator over the deployment transaction IDs, for all deployments.
    pub fn deployment_transaction_ids(&self) -> impl '_ + Iterator<Item = Cow<'_, N::TransactionID>> {
        self.storage.id_map().keys_confirmed()
    }

    /// Returns an iterator over the program IDs, for all deployments.
    pub fn program_ids(&self) -> impl '_ + Iterator<Item = Cow<'_, ProgramID<N>>> {
        self.storage.id_map().values_confirmed().map(|id| match id {
            Cow::Borrowed(id) => Cow::Borrowed(id),
            Cow::Owned(id) => Cow::Owned(id),
        })
    }

    /// Returns an iterator over the programs, for all deployments.
    pub fn programs(&self) -> impl '_ + Iterator<Item = Cow<'_, Program<N>>> {
        self.storage.program_map().values_confirmed().map(|program| match program {
            Cow::Borrowed(program) => Cow::Borrowed(program),
            Cow::Owned(program) => Cow::Owned(program),
        })
    }

    /// Returns an iterator over the `((program ID, function name, edition), verifying key)`, for all deployments.
    pub fn verifying_keys(
        &self,
    ) -> impl '_ + Iterator<Item = (Cow<'_, (ProgramID<N>, Identifier<N>, u16)>, Cow<'_, VerifyingKey<N>>)> {
        self.storage.verifying_key_map().iter_confirmed()
    }

    /// Returns an iterator over the `((program ID, function name, edition), certificate)`, for all deployments.
    pub fn certificates(
        &self,
    ) -> impl '_ + Iterator<Item = (Cow<'_, (ProgramID<N>, Identifier<N>, u16)>, Cow<'_, Certificate<N>>)> {
        self.storage.certificate_map().iter_confirmed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::helpers::memory::DeploymentMemory;

    #[test]
    fn test_insert_get_remove() {
        let rng = &mut TestRng::default();

        // Sample the deployment transaction.
        let transaction = crate::vm::test_helpers::sample_deployment_transaction(rng);
        let transaction_id = transaction.id();

        // Initialize a new transition store.
        let transition_store = TransitionStore::open(None).unwrap();
        // Initialize a new deployment store.
        let deployment_store = DeploymentMemory::open(transition_store).unwrap();

        // Ensure the deployment transaction does not exist.
        let candidate = deployment_store.get_transaction(&transaction_id).unwrap();
        assert_eq!(None, candidate);

        // Insert the deployment transaction.
        deployment_store.insert(&transaction).unwrap();

        // Retrieve the deployment transaction.
        let candidate = deployment_store.get_transaction(&transaction_id).unwrap();
        assert_eq!(Some(transaction), candidate);

        // Remove the deployment.
        deployment_store.remove(&transaction_id).unwrap();

        // Ensure the deployment transaction does not exist.
        let candidate = deployment_store.get_transaction(&transaction_id).unwrap();
        assert_eq!(None, candidate);
    }

    #[test]
    fn test_find_transaction_id() {
        let rng = &mut TestRng::default();

        // Sample the deployment transaction.
        let transaction = crate::vm::test_helpers::sample_deployment_transaction(rng);
        let transaction_id = transaction.id();
        let program_id = match transaction {
            Transaction::Deploy(_, _, ref deployment, _) => *deployment.program_id(),
            _ => panic!("Incorrect transaction type"),
        };

        // Initialize a new transition store.
        let transition_store = TransitionStore::open(None).unwrap();
        // Initialize a new deployment store.
        let deployment_store = DeploymentMemory::open(transition_store).unwrap();

        // Ensure the deployment transaction does not exist.
        let candidate = deployment_store.get_transaction(&transaction_id).unwrap();
        assert_eq!(None, candidate);

        // Ensure the transaction ID is not found.
        let candidate = deployment_store.find_transaction_id_from_program_id(&program_id).unwrap();
        assert_eq!(None, candidate);

        // Insert the deployment.
        deployment_store.insert(&transaction).unwrap();

        // Find the transaction ID.
        let candidate = deployment_store.find_transaction_id_from_program_id(&program_id).unwrap();
        assert_eq!(Some(transaction_id), candidate);

        // Remove the deployment.
        deployment_store.remove(&transaction_id).unwrap();

        // Ensure the transaction ID is not found.
        let candidate = deployment_store.find_transaction_id_from_program_id(&program_id).unwrap();
        assert_eq!(None, candidate);
    }
}
