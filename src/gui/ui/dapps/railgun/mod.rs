//! Shared helpers for the Railgun dapp flows.

use std::time::Duration;
use tokio::time::sleep;

use anyhow::anyhow;
use zeus_eth::{
   alloy_primitives::{Address, Bytes, U256},
   currency::ERC20Token,
   types::ChainId,
   utils::client::RpcClient,
};
use zeus_railgun::{
   RailgunProvider, rand::SeedableRng, rand_chacha::ChaCha12Rng, transact::TransactionBuilder,
};

use crate::core::ZeusCtx;
use crate::utils::{
   RT,
   simulate::{ForkPrefetch, ForkSim, ForkSimRequest, simulate_on_fork},
};

pub mod merge_notes;
pub mod shield;
pub mod transfer;
pub mod unshield;

pub use merge_notes::MergeNotesWindow;
pub use shield::{BundlerUrl, RailgunMode, ShieldUi};
pub use transfer::{private_merge_notes, private_transfer};
pub use unshield::default_bundler_url;

/// A proved Railgun transaction, reduced to the call that gets broadcast.
pub struct ProvedCall {
   pub calldata: Bytes,
   pub interact_to: Address,
   pub value: U256,
}

/// Prove `tx` and reduce it to its call.
pub async fn prove(
   provider: &mut RailgunProvider<RpcClient>,
   tx: TransactionBuilder,
) -> Result<ProvedCall, anyhow::Error> {
   let mut rng = ChaCha12Rng::from_os_rng();
   let proved = provider.build(tx, &mut rng).await?;

   Ok(ProvedCall {
      calldata: proved.tx_data.data.clone(),
      interact_to: proved.tx_data.to,
      value: proved.tx_data.value,
   })
}

/// Fork-simulate a proved Railgun transaction.
///
/// Local notes the chain already spent revert with "note already spent", which
/// means local state is behind: that schedules a resync and surfaces the failure
/// rather than pretending the transaction is simply invalid.
pub async fn simulate_proved(
   ctx: ZeusCtx,
   chain: ChainId,
   from: Address,
   call: &ProvedCall,
   prefetch: ForkPrefetch,
   gas_limit: Option<u64>,
) -> Result<ForkSim, anyhow::Error> {
   let req = ForkSimRequest {
      from,
      interact_to: call.interact_to,
      call_data: call.calldata.clone(),
      value: call.value,
      gas_limit,
      authorization_list: vec![],
   };

   match simulate_on_fork(ctx.clone(), chain, prefetch, req).await {
      Ok(sim) => Ok(sim),
      Err(e) => {
         if e.to_string().contains("note already spent") {
            resync_railgun_later(ctx, chain);
         }

         Err(anyhow!("Simulation failed: {:?}", e))
      }
   }
}

/// Gate every Railgun operation: supported, enabled, provider ready, not syncing,
/// and synced — scheduling a resync when the local root is invalid.
pub async fn railgun_ready(
   ctx: ZeusCtx,
   chain: ChainId,
) -> Result<RailgunProvider<RpcClient>, anyhow::Error> {
   if !ctx.railgun_is_supported(chain) {
      return Err(anyhow!(
         "Railgun is not supported for the {} network",
         chain.name()
      ));
   }

   if !ctx.is_railgun_enabled(chain.id()) {
      return Err(anyhow!(
         "Railgun is disabled. Enable it in Settings → Railgun."
      ));
   }

   let provider = ctx.get_railgun_provider(chain.id(), false).await?;

   if provider.chain_id() != chain.id() {
      return Err(anyhow!(
         "Railgun provider chain id {} does not match the current chain id {}",
         provider.chain_id(),
         chain.id()
      ));
   }

   if provider.is_syncing().await {
      return Err(anyhow!("Railgun is syncing, try again later"));
   }

   if let Err(e) = ctx.sync_railgun(chain.id(), false).await {
      // If railgun cannot sync error out so we dont allow operations
      let is_invalid_root = ctx.read(|ctx| ctx.railgun_status.is_error_invalid_root(chain.id()));
      if is_invalid_root {
         resync_railgun_later(ctx.clone(), chain);

         return Err(anyhow!(
            "Railgun state is corrupted (Invalid root), resync has started"
         ));
      }

      return Err(anyhow!("Railgun is not synced: {:?}", e));
   }

   Ok(provider)
}

/// Refresh public and private state after a Railgun op.
///
/// Every op leaves the sender's public balances stale and moves private notes, so
/// all wallets' private data is refreshed: a note spent here changes what
/// another wallet of the same seed sees.
///
/// Order matters — `sync_railgun` has to land before `update_private_data`, or the
/// refresh reports the state the chain has already moved past.
pub async fn settle_railgun_op(
   ctx: ZeusCtx,
   chain: ChainId,
   from: Address,
   token: Option<ERC20Token>,
) {
   ctx.write(|ctx| {
      ctx.railgun_status.set_op_in_progress(chain.id(), true);
   });

   let manager = ctx.balance_manager();

   if let Some(token) = token {
      if let Err(e) = manager
         .update_tokens_balance(ctx.clone(), chain.id(), from, vec![token], true)
         .await
      {
         tracing::error!("Error updating token balance: {:?}", e);
      }
   }

   if let Err(e) = manager.update_eth_balance(ctx.clone(), chain.id(), vec![from], true).await {
      tracing::error!("Error updating eth balance: {:?}", e);
   }

   ctx.update_public_data(chain.id(), from);

   if let Err(e) = ctx.sync_railgun(chain.id(), false).await {
      tracing::error!("Error syncing Railgun: {:?}", e);
   }

   for wallet in ctx.get_all_wallets_info() {
      ctx.update_private_data(chain.id(), wallet.address).await;
   }

   ctx.write(|ctx| {
      ctx.railgun_status.set_op_in_progress(chain.id(), false);
   });
}

/// Resync Railgun state a second from now.
///
/// Used when local state disagrees with the chain — an invalid root, or notes the
/// chain already spent. Re-reading immediately races whatever caused the
/// disagreement, so give it a beat.
pub fn resync_railgun_later(ctx: ZeusCtx, chain: ChainId) {
   RT.spawn(async move {
      sleep(Duration::from_secs(1)).await;

      match ctx.resync_railgun(chain.id()).await {
         Ok(_) => tracing::info!(
            "Railgun resynced to valid root for chain {}",
            chain.id()
         ),
         Err(e) => tracing::error!("Error syncing Railgun: {:?}", e),
      }
   });
}

/// The single event in `events`, or `too_many` / `none()`.
///
/// `none` is lazy so a caller can include diagnostics, such as how many logs it
/// scanned, without paying for them on the happy path.
pub fn expect_single_event<T>(
   events: Vec<T>,
   too_many: &str,
   none: impl FnOnce() -> String,
) -> Result<T, anyhow::Error> {
   if events.len() > 1 {
      return Err(anyhow!("{}", too_many));
   }

   events.into_iter().next().ok_or_else(|| anyhow!("{}", none()))
}
