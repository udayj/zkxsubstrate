use frame_support::dispatch::Vec;
use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
pub use pallet_prices_runtime_api::PricesApi as PricesRuntimeApi;
use pallet_support::types::ABRDetails;
use primitive_types::U256;
use sp_api::ProvideRuntimeApi;
use sp_arithmetic::fixed_point::FixedI128;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

#[rpc(client, server)]
pub trait PricesApi<BlockHash> {
	#[method(name = "abr_get_remaining_markets")]
	fn get_remaining_markets(&self, at: Option<BlockHash>) -> RpcResult<Vec<U256>>;

	#[method(name = "abr_get_no_of_batches_for_current_epoch")]
	fn get_no_of_batches_for_current_epoch(&self, at: Option<BlockHash>) -> RpcResult<u64>;

	#[method(name = "abr_get_last_timestamp")]
	fn get_last_abr_timestamp(&self, at: Option<BlockHash>) -> RpcResult<u64>;

	#[method(name = "abr_get_remaining_pay_abr_calls")]
	fn get_remaining_pay_abr_calls(&self, at: Option<BlockHash>) -> RpcResult<u64>;

	#[method(name = "abr_get_next_timestamp")]
	fn get_next_abr_timestamp(&self, at: Option<BlockHash>) -> RpcResult<u64>;

	#[method(name = "abr_get_previous_values")]
	fn get_previous_abr_values(
		&self,
		market_id: U256,
		start_timestamp: u64,
		end_timestamp: u64,
		at: Option<BlockHash>,
	) -> RpcResult<Vec<ABRDetails>>;

	#[method(name = "abr_get_intermediary_value")]
	fn get_intermediary_abr_value(
		&self,
		market_id: U256,
		at: Option<BlockHash>,
	) -> RpcResult<FixedI128>;
}

/// A struct that implements the `PricesApi`.
pub struct PricesPallet<C, Block> {
	// If you have more generics, no need to PricesPallet<C, M, N, P, ...>
	// just use a tuple like PricesPallet<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<Block>,
}

impl<C, Block> PricesPallet<C, Block> {
	/// Create new `PricesPallet` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block> PricesApiServer<<Block as BlockT>::Hash> for PricesPallet<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: PricesRuntimeApi<Block>,
{
	fn get_remaining_markets(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<U256>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		api.get_remaining_markets(at).map_err(runtime_error_into_rpc_err)
	}

	fn get_no_of_batches_for_current_epoch(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<u64> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		api.get_no_of_batches_for_current_epoch(at).map_err(runtime_error_into_rpc_err)
	}

	fn get_last_abr_timestamp(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<u64> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		api.get_last_abr_timestamp(at).map_err(runtime_error_into_rpc_err)
	}

	fn get_remaining_pay_abr_calls(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<u64> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		api.get_remaining_pay_abr_calls(at).map_err(runtime_error_into_rpc_err)
	}

	fn get_next_abr_timestamp(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<u64> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		api.get_next_abr_timestamp(at).map_err(runtime_error_into_rpc_err)
	}

	fn get_previous_abr_values(
		&self,
		market_id: U256,
		start_timestamp: u64,
		end_timestamp: u64,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<ABRDetails>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		api.get_previous_abr_values(at, market_id, start_timestamp, end_timestamp)
			.map_err(runtime_error_into_rpc_err)
	}

	fn get_intermediary_abr_value(
		&self,
		market_id: U256,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<FixedI128> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		api.get_intermediary_abr_value(at, market_id)
			.map_err(runtime_error_into_rpc_err)
	}
}

const RUNTIME_ERROR: i32 = 1;

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
	CallError::Custom(ErrorObject::owned(
		RUNTIME_ERROR,
		"Runtime error",
		Some(format!("{:?}", err)),
	))
	.into()
}
