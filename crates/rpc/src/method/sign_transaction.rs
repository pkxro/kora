use kora_lib::{
    config::ValidationConfig,
    transaction::{decode_b58_transaction, sign_transaction as lib_sign_transaction},
    KoraError,
};
use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SignTransactionRequest {
    pub transaction: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SignTransactionResponse {
    pub signature: String,
    pub signed_transaction: String,
}

pub async fn sign_transaction(
    rpc_client: &Arc<RpcClient>,
    validation: &ValidationConfig,
    request: SignTransactionRequest,
) -> Result<SignTransactionResponse, KoraError> {
    let transaction = decode_b58_transaction(&request.transaction)?;
    let (transaction, signed_transaction) =
        lib_sign_transaction(rpc_client, validation, transaction).await?;

    Ok(SignTransactionResponse {
        signature: transaction.signatures[0].to_string(),
        signed_transaction,
    })
}
