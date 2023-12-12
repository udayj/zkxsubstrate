use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use pallet_support::types::{AccountInfo, FeeRates, MarginInfo, PositionExtended};
pub use pallet_trading_runtime_api::TradingApi as TradingRuntimeApi;
use primitive_types::U256;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

#[rpc(client, server)]
pub trait TradingApi<BlockHash> {
	#[method(name = "trading_get_positions")]
	fn positions(
		&self,
		account_id: U256,
		collateral_id: u128,
		at: Option<BlockHash>,
	) -> RpcResult<Vec<PositionExtended>>;

	#[method(name = "trading_get_margin_info")]
	fn get_margin_info(
		&self,
		account_id: U256,
		collateral_id: u128,
		at: Option<BlockHash>,
	) -> RpcResult<MarginInfo>;

	#[method(name = "trading_get_account_info")]
	fn get_account_info(
		&self,
		account_id: U256,
		collateral_id: u128,
		at: Option<BlockHash>,
	) -> RpcResult<AccountInfo>;

	#[method(name = "trading_get_account_list")]
	fn get_account_list(
		&self,
		start_index: u128,
		end_index: u128,
		at: Option<BlockHash>,
	) -> RpcResult<Vec<U256>>;

	#[method(name = "trading_get_fee")]
	fn get_fee(
		&self,
		account_id: U256,
		market_id: U256,
		at: Option<BlockHash>,
	) -> RpcResult<(FeeRates, u64)>;
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
		account_id: U256,
		collateral_id: u128,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<PositionExtended>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		api.positions(at, account_id, collateral_id).map_err(runtime_error_into_rpc_err)
	}

	fn get_margin_info(
		&self,
		account_id: U256,
		collateral_id: u128,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<MarginInfo> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		api.get_margin_info(at, account_id, collateral_id)
			.map_err(runtime_error_into_rpc_err)
	}

	fn get_account_info(
		&self,
		account_id: U256,
		collateral_id: u128,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<AccountInfo> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		api.get_account_info(at, account_id, collateral_id)
			.map_err(runtime_error_into_rpc_err)
	}

	fn get_account_list(
		&self,
		start_index: u128,
		end_index: u128,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<U256>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		api.get_account_list(at, start_index, end_index)
			.map_err(runtime_error_into_rpc_err)
	}

	fn get_fee(
		&self,
		account_id: U256,
		market_id: U256,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<(FeeRates, u64)> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		api.get_fee(at, account_id, market_id).map_err(runtime_error_into_rpc_err)
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
