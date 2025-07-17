use anchor_lang::prelude::*;
pub mod constants;
pub mod context;
pub mod states;
use context::*;
use states::*;

declare_id!("Gt2gt87crJPt9Y3FbnVkAfdwxQ9cVVTE8mhAH6PKVHm7");

#[program]
pub mod core {
    use super::*;

    //1. Initialize the program
    //2. Initialize the device registry
    //3. Initialize the campaign registry
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    //1. Helps user order a new device
    //2. Get them the device id from our device registry (happens off chain )
    // 3. Attach the device id to the user's wallet address and change the state of the device to "ordered"
    // 4. Emit an event to the user that the device has been ordered
    //5 . In future we might use webhooks to check if the device is ordered or not and then take the actions
    pub fn get_device(ctx: Context<GetDevice>, device_id: u32) -> Result<()> {
        let ad_provider = &mut ctx.accounts.ad_provider;
        ad_provider.devices.push(Soulboard {
            device_id,
            device_state: DeviceState::Available,
        });
        emit!(DeviceOrdered {
            device_id,
            device_state: DeviceState::Available,
        });
        Ok(())
    }

    //1. Get the device id given to the  wallet address
    //2. If no device id is found tell them to order a soulboard
    //2. attach the device id to the details shared by the user
    //3. put it public
    pub fn register_provider(ctx: Context<RegisterProvider>) -> Result<()> {
        let ad_provider = &mut ctx.accounts.ad_provider;
        ad_provider.authority = ctx.accounts.authority.key();
        ad_provider.devices = Vec::new();
        emit!(ProviderRegistered {
            authority: ctx.accounts.authority.key(),
        });
        Ok(())
    }

    // Advertisers can create campaigns and give details about the campaign
    // Book a device for running the campaign
    //Allocate budget to the campaign (Which can be added later as well )
    // Emit an event to the advertiser that the campaign has been created
    pub fn create_campaign(ctx: Context<CreateCampaign>) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        campaign.authority = ctx.accounts.authority.key();
        campaign.campaign_id = 0;
        campaign.campaign_name = "".to_string();
        campaign.campaign_description = "".to_string();
        campaign.campaign_budget = 0;
        campaign.campaign_status = CampaignStatus::Active;
        campaign.campaign_providers = Vec::new();
        campaign.campaign_locations = Vec::new();
        emit!(CampaignCreated {
            authority: ctx.accounts.authority.key(),
        });
        Ok(())
    }

    //Advertisers can add budget to the campaign
    //Emit an event to the advertiser that the budget has been added
    pub fn add_budget(ctx: Context<AddBudget>) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        //TODO: Add budget to the campaign
        emit!(BudgetAdded {
            authority: ctx.accounts.authority.key(),
        });
        Ok(())
    }

    //Advertisers can add location to the campaign
    //changes the state of the location to "booked"
    //Emit an event to the advertiser that the location has been added
    pub fn add_location(ctx: Context<AddLocation>) -> Result<()> {
        Ok(())
    }

    //Advertisers can remove location from the campaign
    //changes the state of the location to "available"
    //Emit an event to the advertiser that the location has been removed
    pub fn remove_location(ctx: Context<RemoveLocation>) -> Result<()> {
        Ok(())
    }
}
