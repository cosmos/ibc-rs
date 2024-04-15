use tendermint_light_client_verifier::ProdVerifier;

/// Specifies the Verifier interface that hosts must adhere to when customizing
/// Tendermint client verification behaviour.
///
/// For users who require custom verification logic, i.e., in situations when
/// the Tendermint `ProdVerifier` doesn't provide the desired outcome, users
/// should define a custom verifier struct as a unit struct and then implement
/// `TmVerifier` for it. Note that the custom verifier does need to also
/// implement the `tendermint_light_client_verifier::Verifier` trait.
///
/// In order to wire up the custom verifier, the `verify_client_message` method
/// on the `ClientStateValidation` trait must be implemented. The simplest way
/// to do this is to import and call the standalone `verify_client_message`
/// function located in the `ibc::clients::tendermint::client_state` module,
/// passing in your custom verifier type as its `verifier` parameter. The rest
/// of the methods in the `ClientStateValidation` trait can be implemented by
/// importing and calling their analogous standalone version from the
/// `tendermint::client_state` module, unless bespoke logic is desired for any
/// of those functions.
pub trait TmVerifier {
    type Verifier: tendermint_light_client_verifier::Verifier;

    fn verifier(&self) -> Self::Verifier;
}

/// The default verifier for IBC clients, the Tendermint light client
/// ProdVerifier, for those users who don't require custom verification logic.
pub struct DefaultVerifier;

impl TmVerifier for DefaultVerifier {
    type Verifier = ProdVerifier;

    fn verifier(&self) -> Self::Verifier {
        ProdVerifier::default()
    }
}
