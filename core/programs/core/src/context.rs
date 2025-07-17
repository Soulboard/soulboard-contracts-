use crate::states::*;
use crate::constants::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct RegisterProvider<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init , 
        payer = authority,
        space=ANCHOR_DISCRIMINATOR_SIZE + AdProvider::INIT_SPACE,
        seeds = [b"ad_provider".as_ref() , authority.key().as_ref() ],
        bump
    )]
    pub ad_provider: Account<'info, AdProvider>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetDevice<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"ad_provider".as_ref() , authority.key().as_ref()],
        bump
    )]
    pub ad_provider: Account<'info, AdProvider>,

    pub system_program: Program<'info, System>,

}

#[derive(Accounts)]
pub struct CreateCampaign<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space=ANCHOR_DISCRIMINATOR_SIZE + Campaign::INIT_SPACE,
        seeds = [b"campaign".as_ref() , authority.key().as_ref()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddBudget<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"campaign".as_ref() , authority.key().as_ref()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddLocation<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"campaign".as_ref() , authority.key().as_ref()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveLocation<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"campaign".as_ref() , authority.key().as_ref()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    pub system_program: Program<'info, System>,
}
