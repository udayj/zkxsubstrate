use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
pub use pallet_prices_runtime_api::PricesApi as PricesRuntimeApi;
use pallet_support::types::ABRState;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc(client, server)]
pub trait PricesApi<BlockHash> {
	#[method(name = "get_abr_state")]
	fn get_abr_state(&self, at: Option<BlockHash>) -> RpcResult<ABRState>;
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
	fn get_abr_state(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<ABRState> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		api.get_abr_state(at).map_err(runtime_error_into_rpc_err)
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
