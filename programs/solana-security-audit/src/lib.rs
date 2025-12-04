use anchor_lang::prelude::*;
pub mod instructions;
pub use instructions::*;

pub mod errors;
pub use errors::*;


declare_id!("AQFY6bi1gX2LHSrxSKEUuuq8zWWgfN8XPb8pXyZeuYvg");

#[program]
pub mod solana_security_audit {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn initialize_launch(
        ctx: Context<InitializeLaunch>,
        args: InitializeLaunchArgs,
    ) -> Result<()> {
        ctx.accounts.validate(&args)?;
        InitializeLaunch::handle(ctx, args)
    }

}

#[derive(Accounts)]
pub struct Initialize {}


#[account]
pub struct Launch {
    pub launch_signer: Pubkey,
}