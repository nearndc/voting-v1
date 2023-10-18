use std::cmp::min;

use near_sdk::serde::Serialize;

use crate::*;

/// This is format of output via JSON for the proposal.
#[derive(Serialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalOutput {
    /// Id of the proposal.
    pub id: u32,
    #[serde(flatten)]
    pub proposal: Proposal,
}

/// This is format of output via JSON for the config.
#[derive(Serialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct ConfigOutput {
    pub prop_counter: u32,
    pub bond: U128,
    pub threshold: u32,
    pub end_time: u64,
    pub voting_duration: u64,
    pub iah_registry: AccountId,
    pub community_treasury: AccountId,
}

#[near_bindgen]
impl Contract {
    /**********
     * QUERIES
     **********/

    /// Returns all proposals
    /// Get proposals in paginated view.
    pub fn get_proposals(&self, from_index: u32, limit: u32) -> Vec<ProposalOutput> {
        (from_index..=min(self.prop_counter, from_index + limit))
            .filter_map(|id| {
                self.proposals
                    .get(&id)
                    .map(|proposal| ProposalOutput { id, proposal })
            })
            .collect()
    }

    /// Get specific proposal.
    pub fn get_proposal(&self, id: u32) -> Option<ProposalOutput> {
        self.proposals
            .get(&id)
            .map(|proposal| ProposalOutput { id, proposal })
    }

    pub fn number_of_proposals(&self) -> u32 {
        self.prop_counter
    }

    pub fn config(&self) -> ConfigOutput {
        ConfigOutput {
            prop_counter: self.prop_counter,
            bond: U128(self.bond),
            threshold: self.threshold,
            end_time: self.end_time,
            voting_duration: self.voting_duration,
            iah_registry: self.iah_registry.to_owned(),
            community_treasury: self.community_treasury.to_owned(),
        }
    }
}
