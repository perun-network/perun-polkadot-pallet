use crate::*;

pub trait AppRegistry {
	fn valid_transition<T: pallet::Config>(
		params: &ParamsOf<T>,
		from: &StateOf<T>,
		to: &StateOf<T>,
		signer: ParticipantIndex,
	) -> bool;

	fn transition_weight<T: pallet::Config>(
		params: &ParamsOf<T>,
	) -> Weight;
}
