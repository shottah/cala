mod config;

pub use config::*;

use crate::{transaction::CalaTransactions, tx_template::CalaTxTemplates};

use super::{account::*, journal::*};

#[napi]
pub struct CalaLedger {
  inner: cala_ledger::CalaLedger,
}

#[napi]
impl CalaLedger {
  #[napi(factory)]
  pub async fn connect(config: CalaLedgerConfig) -> napi::Result<Self> {
    use cala_ledger::CalaLedgerConfig as Config;
    let mut builder = Config::builder();
    builder.pg_con(config.pg_con).exec_migrations(true);
    if let Some(n) = config.max_connections {
      builder.max_connections(n);
    }
    let config = builder.build().map_err(crate::generic_napi_error)?;
    // `CalaLedger::init`'s future holds `&PgPool` across awaits and is only
    // *morally* `Send` (rust#100013), so napi's `Send`-requiring async codegen
    // can't await it directly. Run init via `block_on` (which imposes no `Send`
    // bound) on napi's own tokio runtime handle from a blocking thread:
    // `block_on` off a worker thread is allowed, and using napi's handle keeps
    // the pool's runtime affinity correct for the other async methods.
    let handle = tokio::runtime::Handle::current();
    let inner =
      tokio::task::spawn_blocking(move || handle.block_on(cala_ledger::CalaLedger::init(config)))
        .await
        .map_err(crate::generic_napi_error)?
        .map_err(crate::generic_napi_error)?;
    Ok(Self { inner })
  }

  #[napi]
  pub fn accounts(&self) -> napi::Result<CalaAccounts> {
    Ok(CalaAccounts::new(self.inner.accounts()))
  }

  #[napi]
  pub fn journals(&self) -> napi::Result<CalaJournals> {
    Ok(CalaJournals::new(self.inner.journals()))
  }

  #[napi]
  pub fn tx_templates(&self) -> napi::Result<CalaTxTemplates> {
    Ok(CalaTxTemplates::new(self.inner.tx_templates()))
  }

  #[napi]
  pub fn transactions(&self) -> napi::Result<CalaTransactions> {
    Ok(CalaTransactions::new(
      self.inner.transactions(),
      &self.inner,
    ))
  }

}
