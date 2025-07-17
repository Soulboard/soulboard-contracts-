use crate::states::*;
use crate::constants::*;
use anchor_lang::prelude::*;

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
#[instruction(campaign_id: u32)]
pub struct CreateCampaign<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space=ANCHOR_DISCRIMINATOR_SIZE + Campaign::INIT_SPACE,
        seeds = [b"campaign".as_ref() , authority.key().as_ref(), &campaign_id.to_le_bytes()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(campaign_id: u32)]
pub struct AddBudget<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"campaign".as_ref() , authority.key().as_ref(), &campaign_id.to_le_bytes()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(campaign_id: u32, location: Pubkey, device_id: u32)]
pub struct AddLocation<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"campaign".as_ref() , authority.key().as_ref(), &campaign_id.to_le_bytes()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        seeds = [b"ad_provider".as_ref(), location.as_ref()],
        bump
    )]
    pub ad_provider: Account<'info, AdProvider>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(campaign_id: u32, location: Pubkey, device_id: u32)]
pub struct RemoveLocation<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"campaign".as_ref() , authority.key().as_ref(), &campaign_id.to_le_bytes()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        seeds = [b"ad_provider".as_ref(), location.as_ref()],
        bump
    )]
    pub ad_provider: Account<'info, AdProvider>,

    pub system_program: Program<'info, System>,
}
