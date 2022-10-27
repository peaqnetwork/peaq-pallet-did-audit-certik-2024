use std::sync::Arc;
use std::convert::From;

use codec::{Decode, Encode};
use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use peaq_pallet_did::structs::Attribute;
pub use peaq_pallet_did_runtime_api::PeaqDIDApi as PeaqDIDRuntimeApi;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use sp_core::Bytes;
use serde::{Deserialize, Serialize};


#[derive(
	Clone, Encode, Decode, Serialize, Deserialize
)]
pub struct RPCAttribute<BlockNumber, Moment> {
	pub name: Bytes,
	pub value: Bytes,
	pub validity: BlockNumber,
	pub created: Moment,
}

impl<BlockNumber, Moment> From<Attribute::<BlockNumber, Moment>> for RPCAttribute<BlockNumber, Moment> {
    fn from(item: Attribute::<BlockNumber, Moment>) -> Self {
        RPCAttribute {
            name: item.name.into(),
            value: item.value.into(),
            validity: item.validity,
            created: item.created,
        }
    }
}

#[rpc]
pub trait PeaqDIDApi<BlockHash, AccountId, BlockNumber, Moment> {
	#[rpc(name = "peaqdid_readAttributes")]
	fn read_attributes(&self, did_account: AccountId, name: Bytes, at: Option<BlockHash>) -> 
        Result<Option<RPCAttribute<BlockNumber, Moment>>>;
}

/// A struct that implements the [`PeaqDIDApi`].
pub struct PeaqDID<C, B> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<B>,
}

impl<C, B> PeaqDID<C, B> {
	/// Create new `Oracle` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		PeaqDID {
			client,
			_marker: Default::default(),
		}
	}
}

pub enum Error {
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
		}
	}
}


impl<C, Block, AccountId, BlockNumber, Moment> PeaqDIDApi<<Block as BlockT>::Hash, AccountId, BlockNumber, Moment> for PeaqDID<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: PeaqDIDRuntimeApi<Block, AccountId, BlockNumber, Moment>,
	AccountId: Codec,
	BlockNumber: Codec,
	Moment: Codec,
{
	fn read_attributes(&self, did_account: AccountId, name: Bytes, at: Option<<Block as BlockT>::Hash>) -> 
        Result<Option<RPCAttribute<BlockNumber, Moment>>>
    {
   		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		));
        api.read(&at, did_account, name.to_vec()).map(|o| {
            o.map(|item| RPCAttribute::from(item))
        }).map_err(|e| RpcError {
    		code: ErrorCode::ServerError(Error::RuntimeError.into()),
    		message: "Unable to get value.".into(),
    		data: Some(format!("{:?}", e).into()),
    	})
    }
}
