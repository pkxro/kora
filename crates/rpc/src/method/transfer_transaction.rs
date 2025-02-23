use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, message::Message, pubkey::Pubkey,
    system_instruction, transaction::Transaction,
};
use std::{str::FromStr, sync::Arc};
use utoipa::ToSchema;

use kora_lib::{
    config::ValidationConfig, constant::NATIVE_SOL, get_signer,
    transaction::validator::TransactionValidator, KoraError, Signer as _,
};

use crate::{
    config::ValidationConfig,
    error::KoraError,
    token::{token_keg::TokenKeg, TokenInterface},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct TransferTransactionRequest {
    pub amount: u64,
    pub token: String,
    pub source: String,
    pub destination: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TransferTransactionResponse {
    pub transaction: String,
    pub message: String,
    pub blockhash: String,
}

pub async fn transfer_transaction(
    rpc_client: &Arc<RpcClient>,
    validation: &ValidationConfig,
    request: TransferTransactionRequest,
) -> Result<TransferTransactionResponse, KoraError> {
    let signer = get_signer()?;
    let fee_payer = signer.solana_pubkey();

    let validator = TransactionValidator::new(fee_payer, validation)?;

    let source = Pubkey::from_str(&request.source)
        .map_err(|e| KoraError::ValidationError(format!("Invalid source address: {}", e)))?;
    let destination = Pubkey::from_str(&request.destination)
        .map_err(|e| KoraError::ValidationError(format!("Invalid destination address: {}", e)))?;
    let token_mint = Pubkey::from_str(&request.token)
        .map_err(|e| KoraError::ValidationError(format!("Invalid token address: {}", e)))?;

    // manually check disallowed account because we're creating the message
    if validator.is_disallowed_account(&source) {
        return Err(KoraError::InvalidTransaction(format!(
            "Source account {} is disallowed",
            source
        )));
    }

    if validator.is_disallowed_account(&destination) {
        return Err(KoraError::InvalidTransaction(format!(
            "Destination account {} is disallowed",
            destination
        )));
    }

    let mut instructions = vec![];

    // Handle native SOL transfers
    if request.token == NATIVE_SOL {
        instructions.push(system_instruction::transfer(&source, &destination, request.amount));
    } else {
        // Handle wrapped SOL and other SPL tokens
        validator.validate_token_mint(&token_mint)?;

        let mint_data = TokenKeg::get_mint_data(rpc_client, &token_mint)
            .await
            .map_err(|e| KoraError::ValidationError(format!("Failed to get mint data: {}", e)))?;
        let source_ata = TokenKeg::get_associated_account_address(&source, &token_mint);
        let dest_ata = TokenKeg::get_associated_account_address(&destination, &token_mint);

        let _ = TokenKeg::get_token_account_data(rpc_client, &source_ata)
            .await
            .map_err(|e| KoraError::ValidationError(format!("Invalid source token account: {}", e)))?;

        if rpc_client.get_account(&dest_ata).await.is_err() {
            instructions.push(TokenKeg::create_associated_account_instruction(
                &fee_payer,
                &destination,
                &token_mint,
            )?);
        }

        instructions.push(
            TokenKeg::create_transfer_instruction(
                &source_ata,
                &dest_ata,
                &source,
                request.amount,
                mint_data.decimals,
            )
            .await?,
        );
    }

    let blockhash =
        rpc_client.get_latest_blockhash_with_commitment(CommitmentConfig::finalized()).await?;

    let message = Message::new_with_blockhash(&instructions, Some(&fee_payer), &blockhash.0);
    let mut transaction = Transaction::new_unsigned(message);

    // validate transaction before signing
    validator.validate_transaction(&transaction)?;

    let signature = signer.sign_solana(&transaction.message_data()).await?;
    transaction.signatures[0] = signature;

    let serialized = bincode::serialize(&transaction)?;
    let encoded = bs58::encode(serialized).into_string();

    Ok(TransferTransactionResponse {
        transaction: encoded,
        message: bs58::encode(transaction.message.serialize()).into_string(),
        blockhash: blockhash.0.to_string(),
    })
}
