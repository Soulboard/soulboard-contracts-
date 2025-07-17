use anchor_lang::prelude::*;
use anchor_lang::system_program;
pub mod constants;
pub mod context;
pub mod states;
use context::*;
use states::*;

declare_id!("Gt2gt87crJPt9Y3FbnVkAfdwxQ9cVVTE8mhAH6PKVHm7");

#[program]
pub mod soulboard_core {
    use super::*;

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

    // Advertisers can create campaigns and give details about the campaign
    // Book a device for running the campaign
    //Allocate budget to the campaign (Which can be added later as well )
    // Emit an event to the advertiser that the campaign has been created
    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        campaign_id: u32,
        campaign_name: String,
        campaign_description: String,
    ) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        campaign.authority = ctx.accounts.authority.key();
        campaign.campaign_id = campaign_id;
        campaign.campaign_name = campaign_name;
        campaign.campaign_description = campaign_description;
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
    pub fn add_budget(ctx: Context<AddBudget>, _campaign_id: u32, amount: u64) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.authority.to_account_info(),
                to: campaign.to_account_info(),
            },
        );
        system_program::transfer(cpi_context, amount)?;

        campaign.campaign_budget += amount;
        emit!(BudgetAdded {
            authority: ctx.accounts.authority.key(),
        });
        Ok(())
    }

    //Advertisers can add location to the campaign
    //changes the state of the location to "booked"
    //Emit an event to the advertiser that the location has been added
    pub fn add_location(
        ctx: Context<AddLocation>,
        campaign_id: u32,
        location: Pubkey,
        device_id: u32,
    ) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        let ad_provider = &mut ctx.accounts.ad_provider;

        //Find the device and change its state to booked
        let device = ad_provider
            .devices
            .iter_mut()
            .find(|d| d.device_id == device_id)
            .ok_or(ErrorCode::DeviceNotFound)?;

        require!(
            device.device_state == DeviceState::Available,
            ErrorCode::DeviceNotAvailable
        );

        device.device_state = DeviceState::Booked;

        campaign.campaign_providers.push(ad_provider.authority);
        campaign.campaign_locations.push(location);

        emit!(LocationAdded {
            campaign_id,
            location,
        });

        Ok(())
    }

    //Advertisers can remove location from the campaign
    //changes the state of the location to "available"
    //Emit an event to the advertiser that the location has been removed
    pub fn remove_location(
        ctx: Context<RemoveLocation>,
        campaign_id: u32,
        location: Pubkey,
        device_id: u32,
    ) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        let ad_provider = &mut ctx.accounts.ad_provider;

        //Find the device and change its state to available
        let device = ad_provider
            .devices
            .iter_mut()
            .find(|d| d.device_id == device_id)
            .ok_or(ErrorCode::DeviceNotFound)?;

        require!(
            device.device_state == DeviceState::Booked,
            ErrorCode::DeviceNotBooked
        );

        device.device_state = DeviceState::Available;

        //Remove the provider and location from the campaign
        campaign
            .campaign_providers
            .retain(|&p| p != ad_provider.authority);
        campaign.campaign_locations.retain(|&l| l != location);

        emit!(LocationRemoved {
            campaign_id,
            location,
        });

        Ok(())
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Device not found")]
    DeviceNotFound,
    #[msg("Device not available")]
    DeviceNotAvailable,
    #[msg("Device not booked")]
    DeviceNotBooked,
}
