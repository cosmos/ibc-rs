# ADR 006: Upgrade client implementation

## Changelog

* 2023-01-25 Initial Proposal
* 2023-01-30 [Accepted](https://github.com/cosmos/ibc-rs/pull/383)
* 2023-05-10 Finalize Implementation

## Context

The ability to upgrade is a crucial feature for IBC-connected chains, which
enables them to evolve and improve without limitations. The IBC module may not
be affected by some upgrades, but some may require the relevant client be
upgraded as well to keep high-value connections to other chains secure. For
having this capability, chains that implement `IBC-rs` may bring various
concerns and characteristics than Tendermint-based chains leading to different
ways for upgrading their clients. However there are general rules that apply to
all and can serve as a framework for any implementation. This record aims to
justify the chain-wide logic behind upgrading light clients, list requisites for
validation and execution steps, determine the boundary between basic and
upgrade-specific validations by an IBC handler, and explain Tendermint's upgrade
client implementation within the
[ics07_tendermint](../../ibc-clients/ics07-tendermint).

## Decision

In this section, we first introduce the rules that are mainly derived from the
IBC protocol, and the review of IBC-go implementation. Next, we will provide a
detailed outline of the upgrade client process for Tendermint chains. Finally,
we will explain how we have implemented this rationale in IBC-rs.

### Chain-wide Upgrade Rules

#### Chain supports IBC client upgrades?

* IBC currently **ONLY** supports planned upgrades that are committed to in
  advance by the upgrading chain, in order for counterparty clients to maintain
  their connections securely.
* An IBC upgrade **MUST** be performed if upgrading an IBC-connected chain
  breaks the counterparty IBC client.
* There **MUST** be a proof verification process to check upgraded client and
  consensus states against the host chain's state.
* Chain upgrades **MUST NOT** result in changing
  [ClientState](../../ibc-core/ics02-client/context/src/client_state.rs)
  or
  [ConsensusState](../../ibc-core/ics02-client/context/src/consensus_state.rs)
  implementations
* It is **UP TO** the chain's architecture how upgraded client and consensus
  states are committed, either through decentralized approaches, like governance
  or centralized methods, like a multisig account, etc.
* Upon commitment, chain's store **MUST** contain upgraded client and consensus
  states, which can be retrieved using respective upgrade paths

#### IBC Handler accepts upgrade?

After ensuring that the chain upgrades is supported by IBC, the general
validation and execution steps that apply to all chains are as follows. The
criteria for classifying a validation as basic or upgrade-specific was whether
that IBC handler can perform that check just using its own contextual data,
which are available upon calling interfaces provided by `ClientState` and
`ConsensusState` traits, like `is_frozen()`, `latest_height()`, etc.

* Latest client state **MUST NOT** be frozen
* Received update message including verification proofs **MUST** successfully be
  decoded into the domain types
* Clients **MUST** only accept upgrades at the planned last height of the
  current revision which is somehow encoded in the proof verification process.
  This prevents premature upgrades, as the counterparty may cancel or modify
  upgrade plans before the last planned height.
* Latest consensus state **MUST** be within the trusting period of the latest
  client state, which for clients without a trusting period is not applicable.

Any other requisite beyond the above rules are considered client-specific. Next,
we go through the upgrade process for Tendermint clients to justify the logic
and illustrate an example of client-specific upgrade.

### Upgrade Tendermint Clients

This section is based on the `IBC-go` procedure for client upgrades with a few
modifications to comply with the `IBC-rs` design patterns.

#### Acceptable Client State Upgrades

Below list enumerates upgrades on Tendermint chains that would break
counterparty IBC Tendermint clients and indicates whether or not the change is
supported by `IBC-rs`:

* S: Supported, P: Partially Supported, U: Unsupported

1. (S) Changing the `ChainId`
2. (S) Changing the `Height` (resetting to 0): as long as chains remember to
   increment the revision number in their chain-id.
3. (S) Changing the `ProofSpecs`: this should be changed if the proof structure
   needed to verify IBC proofs is changed across the upgrade. Ex: Switching from
   an IAVL store, to a SimpleTree Store.
4. (S) Changing the `UpgradePath`: this might involve changing the key under
   which upgraded clients and consensus states are stored in the upgrade store,
   or even migrating the upgrade store itself.
5. (S) Upgrading to a backwards compatible version of IBC
6. (P) Changing the `UnbondingPeriod`: chains may increase the unbonding period
   with no issues. However, decreasing the unbonding period may irreversibly
   break some counterparty clients (Consider the case where the
   `UnbondingPeriod` falls below the `TrustingPeriod`). Thus, it is not
   recommended that chains reduce the unbonding period.
7. (P) Changing the Tendermint LightClient algorithm: Changes to the light
   client algorithm that do not change the
   [ClientState](../../ibc-clients/ics07-tendermint/src/client_state.rs)
   or
   [ConsensusState](../../ibc-clients/ics07-tendermint/src/consensus_state.rs)
   struct abstraction may be supported, provided that the counterparty is also
   upgraded to support the new light client algorithm. Changes that require
   updating the `ClientState` and `ConsensusState` structs themselves are
   theoretically possible by providing a path to translate an older ClientState
   struct into the new ClientState struct; however this is not currently
   implemented.
8. (U) Migrating the IBC store: the store location is negotiated by the
   connection.
9. (U) Upgrading to a non-backwards compatible version of IBC: the version is
   negotiated on connection handshake.
10. (U) Changing parameters that are customizable by relayers like `TrustLevel`
    and `TrustingPeriod`, `max_clock_drift`
  
#### Upgrade Process Step-by-step

An IBC-connected Tendermint chain will take the following steps to completely
upgrade its own chain and counterparty's IBC client. Note that the chain-level
upgrade instruction (1) is not a part of the IBC protocol. It is provided for
the sake of the big picture and as a reference to follow the upgrade process
from the very beginning when a proposal is initiated to when the upgrade message
is entirely handled.

1. Upgrade chain through governance
   1. Create a 02-client
      [UpgradeProposal](https://github.com/cosmos/ibc-go/blob/main/docs/docs/01-ibc/09-proto-docs.md#upgradeproposal)
      with an `UpgradePlan` and a new IBC `ClientState` in the
      `UpgradedClientState` field with the following remarks:
        * The `UpgradePlan` must specify an upgrade height only (no upgrade
          time)
        * The `ClientState` should only include the fields common to all valid
          clients and zero out any client-customizable fields (such as
          `TrustingPeriod`).
   2. Vote on and pass the `UpgradeProposal`
   3. Commit `UpgradedClient` by the upgrade module under the following key:

        ```md
        upgrade/UpgradedIBCState/{upgradeHeight}/upgradedClient
        ```

   4. Commit an initial consensus state by upgrade module on the block right
      before the upgrade height for the next chain under the following key:

        ```md
        upgrade/UpgradedIBCState/{upgradeHeight}/upgradedConsState
        ```

      Notice that since the `UpgradedConsensusState` will not be available at
      the upgrade path prior to this height, relayers cannot submit a valid
      upgrade message as the proof verification would fail

2. Submit an upgrade client message by relayers to the counterparty chain
   1. Wait for the upgrading chain to reach the upgrade height and halt
   2. Query a full node for the proofs of `UpgradedClient` and
      `UpgradedConsensusState` at the last height of the old chain
   3. Update the counterparty client to the last height of the old chain using
      the `UpdateClient` msg
   4. Submit a `MsgUpgradeClient` message to the counterparty chain with the
      `UpgradedClient`, `UpgradedConsensusState` and their respective proofs

3. Process the upgrade message on the counterparty chain upon receiving a
`MsgUpgradeClient` message performs basic validations (BV), upgrade-specific
validations (SV) and lastly execution (E) steps as follows:
   1. (BV) Check that the current client is not frozen
   2. (BV) Check if the latest consensus state is within the trust period
   3. (BV) Check if the message containing proofs decoded successfully
   4. (SV) Verify that the upgradedClient be of a Tendermint `ClientState` type
   5. (SV) Match any Tendermint chain specified parameter in upgraded client
      such as ChainID, UnbondingPeriod, and ProofSpecs with the committed client
   6. (SV) Verify that the upgradedConsensusState be of a Tendermint
      `ConsensusState` type
   7. (SV) Check the height of the committed client state is not greater than
      the latest height of the current client state
   8. (SV) Verify that the upgrading chain did indeed commit to the upgraded
      client state at the upgrade height by provided proof. Note that the
      client-customizable fields must be zeroed out for this check
   9. (SV) Verify that the upgrading chain did indeed commit to the upgraded
      consensus state at the upgrade height by provided proof
   10. (E) Upgrade client to the new client by retaining old client-customizable
      parameters (sent by relayers) such `TrustingPeriod`, `TrustLevel`,
      `MaxClockDrift` and adopt the new chain-specified fields such as
      `UnbondingPeriod`, `ChainId`, `UpgradePath`, etc.
   11. (E) Upgrade consensus state with a stand-in sentinel value for root. Note
      that the upgraded consensus state serves purely as a basis of trust for
      future `UpdateClientMsgs`, and therefore it does not require a root for
      proof verification and it is not used for packet verifications as well.
      That sentinel value serves as a temporary substitute until the root of new
      chain gets available by `UpdateClientMsg`. It is set by module with a
      distinct, easily recognizable value to reduce the risk of bugs. Thereby,
      we do not also set a processed time for this consensus state. To ensure
      the connection can be used for relaying packets, relayers must submit an
      `UpdateClientMsg` with a header from the new chain.

4. Submit an `UpdateClient` msg by a relayer to the counterparty chain with a
   header from the newly upgraded chain

#### Decisions

Whenever the IBC handler receives an `MsgUpgradeClient`, it dispatches the
decoded message to the router and triggers the
[process](../../ibc-core/ics02-client/src/handler/upgrade_client.rs)
function of `upgrade_client` handler, which would go through the steps outlined
in 3rd section of [Upgrade Process Step-by-Step](#upgrade-process-step-by-step).
Just note that the `process` function will be rendered into `validate` and
`execute` functions due to ongoing changes associated with ADR-005, and to align
with that one of the decisions made is to split off the
`verify_upgrade_and_update_state` method of `ClientState` trait into:

* `verify_upgrade_client` method

   ```rust
   fn verify_upgrade_client(
      &self,
      upgraded_client_state: Any,
      upgraded_consensus_state: Any,
      proof_upgrade_client: MerkleProof,
      proof_upgrade_consensus_state: MerkleProof,
      root: &CommitmentRoot,
   ) -> Result<(), ClientError>;
   ```

* And `update_state_with_upgrade_client` method

   ```rust
   fn update_state_with_upgrade_client(
      &self,
      upgraded_client_state: Any,
      upgraded_consensus_state: Any,
   ) -> Result<UpdatedState, ClientError>;
   ```

There is also a need to move away from `upgrade` method of `ClientState` trait
leaving only zeroed out fields through `zero_custom_fields()`. New state
commitment should be done in `update_state_with_upgrade_client` and stored via
`store_client_result` of keeper context, so the upgrade part is no longer
necessary.

Listed below are the code snippets that correspond to the third step of the
previous section as mentioned:

* **Basic Validations**

1. ```rust
   if old_client_state.is_frozen() {
      return Err(ContextError::ClientError(ClientError::ClientFrozen {
         client_id,
      }));
   }
   ```

2. ```rust
   let old_consensus_state = ctx
      .consensus_state(&client_id, &old_client_state.latest_height())
      .map_err(|_| ClientError::ConsensusStateNotFound {
         client_id: client_id.clone(),
         height: old_client_state.latest_height(),
      })?;

   let now = ctx.host_timestamp()?;
   let duration = now
      .duration_since(&old_consensus_state.timestamp())
      .ok_or_else(|| ClientError::InvalidConsensusStateTimestamp {
         time1: old_consensus_state.timestamp(),
         time2: now,
      })?;

   if old_client_state.expired(duration) {
      return Err(ClientError::HeaderNotWithinTrustPeriod {
         latest_time: old_consensus_state.timestamp(),
         update_time: now,
      });
   };
   ```

3. ```rust
   // Decode the proto message into a domain message inside deliver() function
   // include the proofs in the message
   let envelope = decode(message)?;
   ```

* **Chain-specific Validations**

4. ```rust
   let upgraded_tm_client_state = TmClientState::try_from(upgraded_client_state)?;
   ```

5. This check has been done as part of step 4 while creating an instance of
   `UpgradedClientState` in the domain type.

6. ```rust
   let upgraded_tm_cons_state = TmConsensusState::try_from(upgraded_consensus_state)?;
   ```

7. ```rust
   if self.latest_height() >= upgraded_tm_client_state.latest_height() {
      return Err(ClientError::LowUpgradeHeight {
            upgraded_height: self.latest_height(),
            client_height: upgraded_tm_client_state.latest_height(),
      });
   }
   ```

8. ```rust
   let last_height = self.latest_height().revision_height();

   // Construct the merkle path for the client state
   let mut client_upgrade_path = upgrade_path.clone();
   client_upgrade_path.push(ClientUpgradePath::UpgradedClientState(last_height).to_string());

   let client_upgrade_merkle_path = MerklePath {
      key_path: client_upgrade_path,
   };

   upgraded_tm_client_state.zero_custom_fields();

   let mut client_state_value = Vec::new();
   upgraded_client_state
      .encode(&mut client_state_value)
      .map_err(ClientError::Encode)?;

   // Verify the proof of the upgraded client state
   merkle_proof_upgrade_client
      .verify_membership(
         &self.proof_specs,
         root.clone().into(),
         client_upgrade_merkle_path,
         client_state_value,
         0,
      )
      .map_err(ClientError::Ics23Verification)?;
   ```

9. ```rust
   let last_height = self.latest_height().revision_height();

   // Construct the merkle path for the consensus state
   let mut cons_upgrade_path = upgrade_path;
   cons_upgrade_path
      .push(ClientUpgradePath::UpgradedClientConsensusState(last_height).to_string());
   let cons_upgrade_merkle_path = MerklePath {
      key_path: cons_upgrade_path,
   };

   let mut cons_state_value = Vec::new();
   upgraded_consensus_state
      .encode(&mut cons_state_value)
      .map_err(ClientError::Encode)?;

   // Verify the proof of the upgraded consensus state
   merkle_proof_upgrade_cons_state
      .verify_membership(
            &self.proof_specs,
            root.clone().into(),
            cons_upgrade_merkle_path,
            cons_state_value,
            0,
      )
      .map_err(ClientError::Ics23Verification)?;
   ```

* **Executions**

10. ```rust
    let upgraded_tm_client_state = TmClientState::try_from(upgraded_client_state)?;
    let new_client_state = TmClientState::new(
       upgraded_tm_client_state.chain_id,
       self.trust_level,
       self.trusting_period,
       upgraded_tm_client_state.unbonding_period,
       self.max_clock_drift,
       upgraded_tm_client_state.latest_height,
       upgraded_tm_client_state.proof_specs,
       upgraded_tm_client_state.upgrade_path,
       upgraded_tm_client_state.allow_update,
       upgraded_tm_client_state.frozen_height,
    )?;
    ```

11. ```rust
    let upgraded_tm_cons_state = TmConsensusState::try_from(upgraded_consensus_state)?;
    let sentinel_root = "sentinel_root".as_bytes().to_vec();
    let new_consensus_state = TmConsensusState::new(
       sentinel_root.into(),
       upgraded_tm_cons_state.timestamp,
       upgraded_tm_cons_state.next_validators_hash,
    );
    ```

## Status

Accepted

## Consequences

### Positive

* Resolve the issue of unimplemented upgrade client message in `IBC-rs`
* Keep tendermint upgrade client implementation close to the `IBC-go`

### Negative

* This proposal might not cover upgrade edge cases that arise from specific
  chain architectures. Thus, there should be further investigation in such
  cases.

### Neutral

* As a general fact, upgrading processes are tricky by nature, requiring much
  more care from developers and support from IBC maintainers

## References

* [How to Upgrade IBC Chains and their
  Clients](https://github.com/cosmos/ibc-go/blob/main/docs/docs/01-ibc/05-upgrades/01-quick-guide.md)
* [IBC Client Developer Guide to
  Upgrades](https://github.com/cosmos/ibc-go/blob/main/docs/docs/01-ibc/05-upgrades/02-developer-guide.md)
* [cosmos/ibc-go/Issue 445: IBC upgrade plan
  summary](https://github.com/cosmos/ibc/issues/445)
* [cosmos/cosmos-sdk/PR 7367: Upgrade
  Client](https://github.com/cosmos/cosmos-sdk/pull/7367)
* [cosmos/ibc-go/Issue 2501: Create importable workflow for chains to run
  upgrade tests](https://github.com/cosmos/ibc-go/issues/2501)
* [Hermes relayer documentation: Client
  Upgrade](https://hermes.informal.systems/documentation/commands/upgrade/index.html)
