use crate::constants::*;
use crate::states::*;
use anchor_lang::prelude::*;    

declare_program!(oracle);
use oracle::accounts::DeviceFeed;



// Initialize the global provider registry (should be called once)
#[derive(Accounts)]
pub struct InitializeRegistry<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = ANCHOR_DISCRIMINATOR_SIZE + ProviderRegistry::INIT_SPACE,
        seeds = [b"provider_registry"],
        bump
    )]
    pub provider_registry: Account<'info, ProviderRegistry>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterProvider<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = ANCHOR_DISCRIMINATOR_SIZE + AdProvider::INIT_SPACE,
        seeds = [b"ad_provider".as_ref(), authority.key().as_ref()],
        bump
    )]
    pub ad_provider: Account<'info, AdProvider>,

    #[account(
        mut,
        seeds = [b"provider_registry"],
        bump
    )]
    pub provider_registry: Account<'info, ProviderRegistry>,

    #[account(
        init,
        payer = authority,
        space = ANCHOR_DISCRIMINATOR_SIZE + ProviderMetadata::INIT_SPACE,
        seeds = [b"provider_metadata".as_ref(), authority.key().as_ref()],
        bump
    )]
    pub provider_metadata: Account<'info, ProviderMetadata>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetDevice<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"ad_provider".as_ref(), authority.key().as_ref()],
        bump
    )]
    pub ad_provider: Account<'info, AdProvider>,

    #[account(
        mut,
        seeds = [b"provider_metadata".as_ref(), authority.key().as_ref()],
        bump
    )]
    pub provider_metadata: Account<'info, ProviderMetadata>,

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
        space = ANCHOR_DISCRIMINATOR_SIZE + Campaign::INIT_SPACE,
        seeds = [b"campaign".as_ref(), authority.key().as_ref(), &campaign_id.to_le_bytes()],
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
        seeds = [b"campaign".as_ref(), authority.key().as_ref(), &campaign_id.to_le_bytes()],
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
        seeds = [b"campaign".as_ref(), authority.key().as_ref(), &campaign_id.to_le_bytes()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        seeds = [b"ad_provider".as_ref(), location.as_ref()],
        bump
    )]
    pub ad_provider: Account<'info, AdProvider>,

    #[account(
        mut,
        seeds = [b"provider_metadata".as_ref(), location.as_ref()],
        bump
    )]
    pub provider_metadata: Account<'info, ProviderMetadata>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(campaign_id: u32, location: Pubkey, device_id: u32)]
pub struct RemoveLocation<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"campaign".as_ref(), authority.key().as_ref(), &campaign_id.to_le_bytes()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        seeds = [b"ad_provider".as_ref(), location.as_ref()],
        bump
    )]
    pub ad_provider: Account<'info, AdProvider>,

    #[account(
        mut,
        seeds = [b"provider_metadata".as_ref(), location.as_ref()],
        bump
    )]
    pub provider_metadata: Account<'info, ProviderMetadata>,

    pub system_program: Program<'info, System>,
}

// New context for updating campaign performance from oracle
#[derive(Accounts)]
#[instruction(campaign_id: u32, device_id: u32)]
pub struct UpdateCampaignPerformance<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"campaign".as_ref(), authority.key().as_ref(), &campaign_id.to_le_bytes()],
        bump,
        has_one = authority
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(
        seeds = [b"device_feed", &device_id.to_le_bytes()],
        bump = device_feed.bump,
        seeds::program = oracle::ID,
    )]
    pub device_feed: Account<'info, DeviceFeed>,

    pub oracle_program: Program<'info, oracle::program::Oracle>,
}

// New context for calculating fees
#[derive(Accounts)]
#[instruction(campaign_id: u32)]
pub struct CalculateFees<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"campaign".as_ref(), authority.key().as_ref(), &campaign_id.to_le_bytes()],
        bump,
        has_one = authority
    )]
    pub campaign: Account<'info, Campaign>,

    pub system_program: Program<'info, System>,
}

// New context for withdrawing earnings
#[derive(Accounts)]
#[instruction(campaign_id: u32)]
pub struct WithdrawEarnings<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"campaign".as_ref(), campaign.authority.as_ref(), &campaign_id.to_le_bytes()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        seeds = [b"ad_provider".as_ref(), authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub ad_provider: Account<'info, AdProvider>,

    pub system_program: Program<'info, System>,
}

// New context for completing campaign
#[derive(Accounts)]
#[instruction(campaign_id: u32)]
pub struct CompleteCampaign<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"campaign".as_ref(), authority.key().as_ref(), &campaign_id.to_le_bytes()],
        bump,
        has_one = authority
    )]
    pub campaign: Account<'info, Campaign>,

    pub system_program: Program<'info, System>,
}

// New context for querying providers
#[derive(Accounts)]
pub struct QueryProviders<'info> {
    #[account(
        seeds = [b"provider_registry"],
        bump
    )]
    pub provider_registry: Account<'info, ProviderRegistry>,
}

// Context for updating provider information
#[derive(Accounts)]
pub struct UpdateProvider<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"ad_provider".as_ref(), authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub ad_provider: Account<'info, AdProvider>,

    #[account(
        mut,
        seeds = [b"provider_metadata".as_ref(), authority.key().as_ref()],
        bump
    )]
    pub provider_metadata: Account<'info, ProviderMetadata>,

    pub system_program: Program<'info, System>,
}