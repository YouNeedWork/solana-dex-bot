use crate::{constants, jito};

use anyhow::Result;
use raydium_library::amm;
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_request::TokenAccountsFilter,
};
use solana_sdk::{
    instruction::Instruction,
    program_pack::Pack,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};
use spl_token::state::Account;
use std::{str::FromStr, sync::Arc};
use tracing::trace;

use solana_sdk::system_instruction::transfer;

pub struct Trade {
    pub keypair: Arc<Keypair>,
    pub rpc: RpcClient,
}

impl Trade {
    pub fn new(keypair: Keypair, rpc: RpcClient) -> Self {
        Self {
            keypair: Arc::new(keypair),
            rpc,
        }
    }

    pub async fn get_balance(&self) -> Result<u64> {
        self.rpc
            .get_balance(&self.keypair.pubkey())
            .await
            .map_err(|e| anyhow::format_err!(e))
    }

    pub async fn get_spl_balance(&self, mint: &Pubkey) -> Result<u64> {
        let token_accounts = self
            .rpc
            .get_token_accounts_by_owner(
                &self.keypair.pubkey(),
                TokenAccountsFilter::Mint(*mint),
            )
            .await?;

        match token_accounts.first() {
            Some(token_account) => {
                let acount_info = self
                    .rpc
                    .get_account(&Pubkey::from_str(
                        token_account.pubkey.as_str(),
                    )?)
                    .await?;
                let token_account_info = Account::unpack(&acount_info.data)?;
                trace!("Token account info: {:?}", token_account_info);
                Ok(token_account_info.amount)
            }
            None => anyhow::bail!("No token account found"),
        }
    }

    pub fn check_is_token_addr(&self, token: &str) -> bool {
        Pubkey::from_str(token).is_ok()
    }

    pub async fn swap(
        &self, amm_pool_id: Pubkey, input_token_mint: Pubkey,
        output_token_mint: Pubkey, amount_specified: u64, slippage_bps: u64,
        tip: u64, budget: u32,
    ) -> Result<()> {
        let amm_program =
            Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY)?;
        let swap_base_in = true;
        let amm_keys =
            amm::utils::load_amm_keys(&self.rpc, &amm_program, &amm_pool_id)
                .await?;

        let market_keys = amm::openbook::get_keys_for_market(
            &self.rpc,
            &amm_keys.market_program,
            &amm_keys.market,
        )
        .await?;

        // calculate amm pool vault with load data at the same time or use
        // simulate to calculate
        let result = amm::calculate_pool_vault_amounts(
            &self.rpc,
            &amm_program,
            &amm_pool_id,
            &amm_keys,
            &market_keys,
            amm::utils::CalculateMethod::Simulate(self.keypair.pubkey()),
        )
        .await?;

        let direction = if input_token_mint == amm_keys.amm_coin_mint
            && output_token_mint == amm_keys.amm_pc_mint
        {
            amm::utils::SwapDirection::Coin2PC
        } else {
            amm::utils::SwapDirection::PC2Coin
        };

        let other_amount_threshold = amm::swap_with_slippage(
            result.pool_pc_vault_amount,
            result.pool_coin_vault_amount,
            result.swap_fee_numerator,
            result.swap_fee_denominator,
            direction,
            amount_specified,
            swap_base_in,
            slippage_bps,
        )?;

        let mut swap = Swap {
            pre_swap_instructions: vec![],
            post_swap_instructions: vec![],
        };

        let user_source = handle_token_account(
            &mut swap,
            &self.rpc,
            &input_token_mint,
            other_amount_threshold,
            &self.keypair.pubkey(),
            &self.keypair.pubkey(),
        )
        .await?;

        let user_destination = handle_token_account(
            &mut swap,
            &self.rpc,
            &output_token_mint,
            0,
            &self.keypair.pubkey(),
            &self.keypair.pubkey(),
        )
        .await?;

        // build swap instruction
        let build_swap_instruction = amm::instructions::swap(
            &amm_program,
            &amm_keys,
            &market_keys,
            &self.keypair.pubkey(),
            &user_source,
            &user_destination,
            amount_specified,
            other_amount_threshold,
            swap_base_in,
        )?;

        trace!("{:?}", build_swap_instruction);

        //let blockhash = self.rpc.get_latest_blockhash().await?;
        let mut txs = [
            make_compute_budget_ixs(0, budget),
            swap.pre_swap_instructions.clone(),
            vec![build_swap_instruction],
            swap.post_swap_instructions.clone(),
        ]
        .concat();

        let fee = 500000;
        txs.push(transfer(
            &self.keypair.pubkey(),
            &Pubkey::from_str(constants::BOT_FEE_PUBKEY)?,
            fee,
        ));

        use solana_sdk::transaction::{Transaction, VersionedTransaction};

        use solana_program::message::Message;
        let blockhash = self.rpc.get_latest_blockhash().await?;

        let mut message = Message::new(&txs, Some(&self.keypair.pubkey()));
        message.recent_blockhash = blockhash;

        let mut tx = Transaction::new_unsigned(message);

        let signature = self.keypair.sign_message(&tx.message_data());
        tx.replace_signatures(&[(self.keypair.pubkey(), signature)])?;

        //let searcher_client = "";
        let tx = VersionedTransaction::from(tx);

        let sim_res = self.rpc.simulate_transaction(&tx).await?;
        dbg!(sim_res);
        //jito::send_swap_tx([tx], 50000, &self.keypair, searcher_client,
        // &self.rpc); let res = self.rpc.send_and_confirm_transaction(&
        // tx).await?;

        let mut jito_client = jito::get_searcher_client(
            "https://frankfurt.mainnet.block-engine.jito.wtf",
        )
        .await?;

        let mut times = 3;
        while times > 0 {
            if jito::send_swap_tx(
                &mut txs,
                tip,
                &self.keypair,
                &mut jito_client,
                &self.rpc,
            )
            .await
            .is_err()
            {
                times -= 1;
            } else {
                break;
            }
        }

        if times == 0 {
            anyhow::bail!("transfer did not land!")
        }

        tracing::info!("success swap");

        Ok(())
    }
}

pub fn create_ata_token_or_not(
    funding: &Pubkey, mint: &Pubkey, owner: &Pubkey,
) -> Vec<Instruction> {
    vec![
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            funding,
            owner,
            mint,
            &spl_token::id(),
        ),
    ]
}

pub fn close_account(
    close_account: &Pubkey, destination: &Pubkey, close_authority: &Pubkey,
) -> Vec<Instruction> {
    vec![spl_token::instruction::close_account(
        &spl_token::id(),
        close_account,
        destination,
        close_authority,
        &[],
    )
    .unwrap()]
}

pub fn make_compute_budget_ixs(price: u64, max_units: u32) -> Vec<Instruction> {
    vec![
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_price(price),
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(max_units),
    ]
}

pub fn create_init_token(
    token: &Pubkey, seed: &str, mint: &Pubkey, owner: &Pubkey,
    funding: &Pubkey, lamports: u64,
) -> Vec<Instruction> {
    vec![
        solana_sdk::system_instruction::create_account_with_seed(
            funding,
            token,
            owner,
            seed,
            lamports,
            spl_token::state::Account::LEN as u64,
            &spl_token::id(),
        ),
        spl_token::instruction::initialize_account(
            &spl_token::id(),
            token,
            mint,
            owner,
        )
        .unwrap(),
    ]
}

pub fn generate_pub_key(from: &Pubkey, seed: &str) -> Pubkey {
    Pubkey::create_with_seed(from, seed, &spl_token::id()).unwrap()
}

pub struct Swap {
    pre_swap_instructions: Vec<Instruction>,
    post_swap_instructions: Vec<Instruction>,
}

pub async fn handle_token_account(
    swap: &mut Swap, client: &RpcClient, mint: &Pubkey, amount: u64,
    owner: &Pubkey, funding: &Pubkey,
) -> Result<Pubkey> {
    // two cases - an account is a token account or a native account (WSOL)
    if (*mint).to_string() == "So11111111111111111111111111111111111111112" {
        let rent = client
            .get_minimum_balance_for_rent_exemption(
                spl_token::state::Account::LEN,
            )
            .await?;
        let lamports = rent + amount;
        let seed = &Keypair::new().pubkey().to_string()[0..32];
        let token = generate_pub_key(owner, seed);
        let mut init_ixs =
            create_init_token(&token, seed, mint, owner, funding, lamports);
        let mut close_ixs = close_account(&token, owner, owner);
        // swap.signers.push(token);
        swap.pre_swap_instructions.append(&mut init_ixs);
        swap.post_swap_instructions.append(&mut close_ixs);
        Ok(token)
    } else {
        let token = &spl_associated_token_account::get_associated_token_address(
            owner, mint,
        );
        let mut ata_ixs = create_ata_token_or_not(funding, mint, owner);
        swap.pre_swap_instructions.append(&mut ata_ixs);
        Ok(*token)
    }
}
