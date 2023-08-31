use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
pub use pallet_trading_runtime_api::TradingApi as TradingRuntimeApi;
use primitive_types::U256;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;
use zkx_support::types::Position;

#[rpc(client, server)]
pub trait TradingApi<BlockHash> {
	#[method(name = "trading_positions")]
	fn positions(
		&self,
		at: Option<BlockHash>,
		account_id: U256,
		markets: Vec<U256>,
	) -> RpcResult<Vec<Position>>;

	#[method(name = "trading_collateral_to_market")]
	fn collateral_to_market(&self, at: Option<BlockHash>, account_id: U256)
		-> RpcResult<Vec<U256>>;
}

/// A struct that implements the `TemplateApi`.
pub struct TradingPallet<C, Block> {
	// If you have more generics, no need to TemplatePallet<C, M, N, P, ...>
	// just use a tuple like TemplatePallet<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<Block>,
}

impl<C, Block> TradingPallet<C, Block> {
	/// Create new `TemplatePallet` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block> TradingApiServer<<Block as BlockT>::Hash> for TradingPallet<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: TradingRuntimeApi<Block>,
{
	fn positions(
		&self,
		at: Option<<Block as BlockT>::Hash>,
		account_id: U256,
		markets: Vec<U256>,
	) -> RpcResult<Vec<Position>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		let markets: Vec<U256> = api
			.collateral_to_market(at, account_id)
			.map_err(runtime_error_into_rpc_err)
			.unwrap();

		api.positions(at, account_id, markets).map_err(runtime_error_into_rpc_err)
	}

	fn collateral_to_market(
		&self,
		at: Option<<Block as BlockT>::Hash>,
		account_id: U256,
	) -> RpcResult<Vec<U256>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;

		api.collateral_to_market(at, account_id).map_err(runtime_error_into_rpc_err)
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
