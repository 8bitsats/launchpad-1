//! SetTestOraclePrice instruction handler

use {
    crate::{
        oracle::TestOracle,
        state::{
            custody::Custody,
            auction::Auction,
            multisig::{AdminInstruction, Multisig},
        },
    },
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct SetTestOraclePrice<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut, seeds = [b"multisig"], bump = multisig.load()?.bump)]
    pub multisig: AccountLoader<'info, Multisig>,

    #[account(
        mut, 
        seeds = [b"auction", auction.common.name.as_bytes()],
        bump = auction.bump
    )]
    pub auction: Box<Account<'info, Auction>>,

    #[account(
        mut,
        constraint = custody.key() == auction.pricing.custody,
        seeds = [b"custody", custody.mint.as_ref()],
        bump = custody.bump
    )]
    pub custody: Box<Account<'info, Custody>>,

    #[account(
        init_if_needed, payer = admin, space = TestOracle::LEN,
        constraint = oracle_account.key() == custody.oracle_account,
        seeds = [b"oracle_account",
                 custody.mint.as_ref(),
                 auction.key().as_ref()],
        bump
    )]
    pub oracle_account: Box<Account<'info, TestOracle>>,

    system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetTestOraclePriceParams {
    pub price: u64,
    pub expo: i32,
    pub conf: u64,
    pub publish_time: i64,
}

pub fn set_test_oracle_price<'info>(
    ctx: Context<'_, '_, '_, 'info, SetTestOraclePrice<'info>>,
    params: &SetTestOraclePriceParams,
) -> Result<u8> {
    // validate signatures
    let mut multisig = ctx.accounts.multisig.load_mut()?;

    let signatures_left = multisig.sign_multisig(
        &ctx.accounts.admin,
        &Multisig::get_account_infos(&ctx)[1..],
        &Multisig::get_instruction_data(AdminInstruction::SetTestOraclePrice, params)?,
    )?;
    if signatures_left > 0 {
        msg!(
            "Instruction has been signed but more signatures are required: {}",
            signatures_left
        );
        return Ok(signatures_left);
    }

    // update oracle data
    let oracle_account = ctx.accounts.oracle_account.as_mut();
    oracle_account.price = params.price;
    oracle_account.expo = params.expo;
    oracle_account.conf = params.conf;
    oracle_account.publish_time = params.publish_time;

    Ok(0)
}
