use crate::*;

pub trait AppRegistry {
	fn valid_transition<T: pallet::Config>(
		params: &ParamsOf<T>,
		from: &StateOf<T>,
		to: &StateOf<T>,
		signer: ParticipantIndex,
	) -> bool;
}

impl AppRegistry for () {
	fn valid_transition<T: pallet::Config>(
		_params: &ParamsOf<T>,
		_from: &StateOf<T>,
		_to: &StateOf<T>,
		_signer: ParticipantIndex,
	) -> bool {
		return false;
	}
}
