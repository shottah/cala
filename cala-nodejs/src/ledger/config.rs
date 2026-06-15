#[napi(object)]
pub struct CalaLedgerConfig {
  pub pg_con: String,
  pub max_connections: Option<u32>,
}
