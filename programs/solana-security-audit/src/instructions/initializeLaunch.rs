use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use crate::errors::LaunchpadError;
use crate::Launch;

const AVAILABLE_TOKENS: u64 = 1_000_000_000_000; // 1M tokens with 6 decimals

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeLaunchArgs {}

#[derive(Accounts)]
pub struct InitializeLaunch<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + std::mem::size_of::<Launch>(),
        seeds = [b"launch", token_mint.key().as_ref()],
        bump
    )]
    pub launch: Account<'info, Launch>,
    
    /// CHECK: This is the launch signer - should be a PDA but we don't validate it (vulnerability!)
    /// NOTE: Adding PDA constraint to test if UncheckedAccount type is the issue
    /// The vulnerability is that we don't explicitly verify it matches expected PDA in code logic
    // #[account(
    //     seeds = [b"launch_signer", launch.key().as_ref()],
    //     bump
    // )]
    pub launch_signer: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(
        init,
        payer = creator,
        associated_token::mint = token_mint,
        associated_token::authority = launch_signer
    )]
    pub token_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeLaunch<'info> {
    pub fn validate(&self, _args: &InitializeLaunchArgs) -> Result<()> {
        msg!("[validate] Starting validation");
        msg!("[validate] Token mint: {}", self.token_mint.key());
        msg!("[validate] Token mint supply: {}", self.token_mint.supply);
        msg!("[validate] Token mint freeze_authority: {:?}", self.token_mint.freeze_authority);
        
        require_eq!(
            self.token_mint.supply,
            0,
            LaunchpadError::SupplyNonZero
        );
        msg!("[validate] Token mint supply check passed");
        
        require!(
            self.token_mint.freeze_authority.is_none(),
            LaunchpadError::FreezeAuthoritySet
        );
        msg!("[validate] Freeze authority check passed");
        msg!("[validate] Validation complete");
        Ok(())
    }

    pub fn handle(ctx: Context<Self>, _args: InitializeLaunchArgs) -> Result<()> {
        msg!("[handle] Starting initializeLaunch handler");
        msg!("[handle] Program ID: {}", ctx.program_id);
        msg!("[handle] Creator: {}", ctx.accounts.creator.key());
        
        let launch_key = ctx.accounts.launch.key();
        msg!("[handle] Launch account key: {}", launch_key);
        msg!("[handle] Token mint: {}", ctx.accounts.token_mint.key());
        msg!("[handle] Token vault: {}", ctx.accounts.token_vault.key());
        msg!("[handle] Launch signer account key: {}", ctx.accounts.launch_signer.key());
        
        // Get bump from Anchor's context (set by PDA constraint)
        // This tests if adding PDA constraint fixes the UncheckedAccount issue
        // let launch_signer_bump = ctx.bumps.launch_signer;
        // let launch_signer_pda = ctx.accounts.launch_signer.key();

        //TODO: Missing validation - should check that ctx.accounts.launch_signer.key() == launch_signer_pda
        // This is the vulnerability: the program doesn't verify launch_signer matches the expected PDA
        // Without this check, an attacker could potentially pass a different account
        
        msg!("[handle] Deriving launch_signer PDA with seeds: [b\"launch_signer\", launch.key()]");
        let (launch_signer, launch_signer_pda_bump) = Pubkey::find_program_address(
            &[b"launch_signer", ctx.accounts.launch.key().as_ref()],
            ctx.program_id,
        );
        msg!("[handle] Derived launch_signer PDA: {}", launch_signer);
        msg!("[handle] Derived launch_signer bump: {}", launch_signer_pda_bump);

        msg!("[handle] Setting launch account data");
        ctx.accounts.launch.set_inner(Launch {
            launch_signer: launch_signer,
        });
        msg!("[handle] Launch account data set successfully");

        // Create signer seeds for PDA
        // Seeds must match the derivation: [b"launch_signer", launch.key(), bump]
        let seeds = &[
            b"launch_signer",
            launch_key.as_ref(),
            &[launch_signer_pda_bump],
        ];
        let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];
        
        msg!("[handle] Created signer seeds:");
        msg!("[handle]   - Seed 0 (\"launch_signer\"): {:?}", b"launch_signer");
        msg!("[handle]   - Seed 1 (launch.key()): {:?}", launch_key.as_ref());
        msg!("[handle]   - Seed 2 (bump): {:?}", &[launch_signer_pda_bump]);

        msg!("[handle] Ready to mint tokens to token_vault");
        msg!("[handle] Launch_signer PDA (expected): {}", launch_signer);
        msg!("[handle] Launch_signer account key (provided): {}", ctx.accounts.launch_signer.key());
        msg!("[handle] Bump: {}", launch_signer_pda_bump);
        msg!("[handle] PDA match check: {}", launch_signer == ctx.accounts.launch_signer.key());
        
        // Verify launch_signer matches the expected PDA
        // This is the check that should be present to prevent the vulnerability
        require_keys_eq!(
            ctx.accounts.launch_signer.key(),
            launch_signer,
            LaunchpadError::InvalidMintAuthority
        );

        msg!("[handle] Preparing CPI to mint {} tokens", AVAILABLE_TOKENS);
        msg!("[handle] CPI context - mint: {}, to: {}, authority: {}", 
            ctx.accounts.token_mint.key(), 
            ctx.accounts.token_vault.key(),
            ctx.accounts.launch_signer.key());
        
        // Mint total tokens to launch token vault
        // Using PDA seeds as signer - we manually derived the PDA and use its seeds
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                MintTo {
                    mint: ctx.accounts.token_mint.to_account_info(),
                    to: ctx.accounts.token_vault.to_account_info(),
                    authority: ctx.accounts.launch_signer.to_account_info(),
                },
                signer_seeds,
            ),
            AVAILABLE_TOKENS,
        )?;

        msg!("[handle] Successfully minted {} tokens to token vault", AVAILABLE_TOKENS);
        msg!("[handle] InitializeLaunch handler completed successfully");
        Ok(())
    }
}