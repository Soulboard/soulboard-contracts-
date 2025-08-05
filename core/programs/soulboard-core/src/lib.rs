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

    // Initialize the global provider registry (call this once when deploying)
    pub fn initialize_registry(ctx: Context<InitializeRegistry>) -> Result<()> {
        let registry = &mut ctx.accounts.provider_registry;
        registry.total_providers = 0;
        registry.providers = Vec::new();

        emit!(RegistryInitialized {
            registry: ctx.accounts.provider_registry.key(),
        });
        Ok(())
    }

    // Register a provider and add to global registry
    pub fn register_provider(
        ctx: Context<RegisterProvider>,
        name: String,
        location: String,
        contact_email: String,
    ) -> Result<()> {
        let ad_provider = &mut ctx.accounts.ad_provider;
        let registry = &mut ctx.accounts.provider_registry;
        let metadata = &mut ctx.accounts.provider_metadata;

        // Check if registry has space for another provider
        require!(
            registry.providers.len() < 50, // MAX_PROVIDERS_IN_REGISTRY
            ErrorCode::RegistryFull
        );

        // Initialize provider account
        ad_provider.authority = ctx.accounts.authority.key();
        ad_provider.devices = Vec::new();
        ad_provider.name = name.clone();
        ad_provider.location = location.clone();
        ad_provider.contact_email = contact_email;
        ad_provider.rating = 50; // Default rating
        ad_provider.total_campaigns = 0;
        ad_provider.is_active = true;
        ad_provider.total_earnings = 0;
        ad_provider.pending_payments = 0;

        // Add to global registry
        registry.providers.push(ctx.accounts.authority.key());
        registry.total_providers += 1;

        // Initialize metadata for faster querying
        metadata.authority = ctx.accounts.authority.key();
        metadata.provider_pda = ctx.accounts.ad_provider.key();
        metadata.name = name.clone();
        metadata.location = location.clone();
        metadata.device_count = 0;
        metadata.available_devices = 0;
        metadata.rating = 50;
        metadata.is_active = true;

        emit!(ProviderRegistered {
            authority: ctx.accounts.authority.key(),
            name,
            location,
        });
        Ok(())
    }

    // Add a device and update metadata
    pub fn get_device(ctx: Context<GetDevice>, device_id: u32) -> Result<()> {
        let ad_provider = &mut ctx.accounts.ad_provider;
        let metadata = &mut ctx.accounts.provider_metadata;

        ad_provider.devices.push(Soulboard {
            device_id,
            device_state: DeviceState::Available,
        });

        // Update metadata
        metadata.device_count += 1;
        metadata.available_devices += 1;

        emit!(DeviceOrdered {
            device_id,
            device_state: DeviceState::Available,
        });

        emit!(ProviderMetadataUpdated {
            authority: ctx.accounts.authority.key(),
            available_devices: metadata.available_devices,
        });

        Ok(())
    }

    // Create campaign with fee structure
    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        campaign_id: u32,
        campaign_name: String,
        campaign_description: String,
        running_days: u32,
        hours_per_day: u32,
        base_fee_per_hour: u64, // in lamports
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
        campaign.running_days = running_days;
        campaign.hours_per_day = hours_per_day;
        campaign.base_fee_per_hour = base_fee_per_hour;
        campaign.platform_fee = 0;
        campaign.total_distributed = 0;
        campaign.campaign_performance = Vec::new();

        emit!(CampaignCreated {
            authority: ctx.accounts.authority.key(),
        });
        Ok(())
    }

    // Add budget (unchanged but now calculates platform fee)
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
        
        // Calculate platform fee (2% of budget)
        campaign.platform_fee = campaign.campaign_budget
            .checked_mul(2)
            .ok_or(ErrorCode::CalculationError)?
            .checked_div(100)
            .ok_or(ErrorCode::CalculationError)?;

        emit!(BudgetAdded {
            authority: ctx.accounts.authority.key(),
        });
        Ok(())
    }

    // Add location and update metadata
    pub fn add_location(
        ctx: Context<AddLocation>,
        campaign_id: u32,
        location: Pubkey,
        device_id: u32,
    ) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        let ad_provider = &mut ctx.accounts.ad_provider;
        let metadata = &mut ctx.accounts.provider_metadata;

        // Find the device and change its state to booked
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

        // Initialize performance tracking for this provider
        campaign.campaign_performance.push(ProviderPerformance {
            provider: ad_provider.authority,
            device_id,
            total_views: 0,
            total_taps: 0,
            calculated_earnings: 0,
            base_fee_earned: 0,
            performance_fee_earned: 0,
        });

        // Update provider metadata
        metadata.available_devices -= 1;
        ad_provider.total_campaigns += 1;

        emit!(LocationAdded {
            campaign_id,
            location,
        });

        emit!(ProviderMetadataUpdated {
            authority: location,
            available_devices: metadata.available_devices,
        });

        Ok(())
    }

    // Remove location and update metadata
    pub fn remove_location(
        ctx: Context<RemoveLocation>,
        campaign_id: u32,
        location: Pubkey,
        device_id: u32,
    ) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        let ad_provider = &mut ctx.accounts.ad_provider;
        let metadata = &mut ctx.accounts.provider_metadata;

        // Find the device and change its state to available
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

        // Remove the provider and location from the campaign
        campaign
            .campaign_providers
            .retain(|&p| p != ad_provider.authority);
        campaign.campaign_locations.retain(|&l| l != location);

        // Remove performance tracking
        campaign.campaign_performance.retain(|p| p.provider != ad_provider.authority);

        // Update metadata
        metadata.available_devices += 1;

        emit!(LocationRemoved {
            campaign_id,
            location,
        });

        emit!(ProviderMetadataUpdated {
            authority: location,
            available_devices: metadata.available_devices,
        });

        Ok(())
    }

    // Update campaign performance by fetching data from oracle
    pub fn update_campaign_performance(
        ctx: Context<UpdateCampaignPerformance>,
        campaign_id: u32,
        device_id: u32,
    ) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        let device_feed = &ctx.accounts.device_feed;

        // Find the performance record for this device
        let performance = campaign
            .campaign_performance
            .iter_mut()
            .find(|p| p.device_id == device_id)
            .ok_or(ErrorCode::DeviceNotFound)?;

        // Update performance metrics from oracle
        performance.total_views = device_feed.total_views;
        performance.total_taps = device_feed.total_taps;

        emit!(PerformanceUpdated {
            campaign_id,
            device_id,
            total_views: performance.total_views,
            total_taps: performance.total_taps,
        });

        Ok(())
    }

    // Calculate and distribute fees based on performance
    pub fn calculate_and_distribute_fees(
        ctx: Context<CalculateFees>,
        campaign_id: u32,
    ) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        
        require!(
            campaign.campaign_status == CampaignStatus::Completed,
            ErrorCode::CampaignNotCompleted
        );

        // Extract all values we need before mutable borrow to avoid borrow checker issues
        let total_campaign_hours = campaign.running_days
            .checked_mul(campaign.hours_per_day)
            .ok_or(ErrorCode::CalculationError)?;

        let num_asps = campaign.campaign_providers.len() as u64;
        let total_hours_u64 = total_campaign_hours as u64;
        let base_fee_per_hour = campaign.base_fee_per_hour;
        let campaign_budget = campaign.campaign_budget;
        let platform_fee = campaign.platform_fee;
        
        // Calculate total base fees for all ASPs
        let total_base_fees = total_hours_u64
            .checked_mul(base_fee_per_hour)
            .ok_or(ErrorCode::CalculationError)?
            .checked_mul(num_asps)
            .ok_or(ErrorCode::CalculationError)?;

        // Calculate distribution fee (remaining budget after base fees and platform fee)
        let available_for_distribution = campaign_budget
            .checked_sub(total_base_fees)
            .ok_or(ErrorCode::InsufficientBudget)?
            .checked_sub(platform_fee)
            .ok_or(ErrorCode::InsufficientBudget)?;

        // Calculate total views across all devices
        let total_views: u64 = campaign.campaign_performance
            .iter()
            .map(|p| p.total_views)
            .sum();

        require!(total_views > 0, ErrorCode::NoViews);

        // Calculate earnings for each ASP
        for performance in campaign.campaign_performance.iter_mut() {
            // Base fee calculation (individual ASP gets paid for all campaign hours)
            let base_fee = total_hours_u64
                .checked_mul(base_fee_per_hour)
                .ok_or(ErrorCode::CalculationError)?;

            // Performance-based distribution
            let performance_share = available_for_distribution
                .checked_mul(performance.total_views)
                .ok_or(ErrorCode::CalculationError)?
                .checked_div(total_views)
                .ok_or(ErrorCode::CalculationError)?;

            // Total before platform fee
            let total_before_platform_fee = base_fee
                .checked_add(performance_share)
                .ok_or(ErrorCode::CalculationError)?;

            // Platform fee (2% of total earnings)
            let provider_platform_fee = total_before_platform_fee
                .checked_mul(2)
                .ok_or(ErrorCode::CalculationError)?
                .checked_div(100)
                .ok_or(ErrorCode::CalculationError)?;

            // Final earnings after platform fee
            let final_earnings = total_before_platform_fee
                .checked_sub(provider_platform_fee)
                .ok_or(ErrorCode::CalculationError)?;

            performance.base_fee_earned = base_fee;
            performance.performance_fee_earned = performance_share;
            performance.calculated_earnings = final_earnings;
        }

        // Calculate total distributed after all calculations are done
        campaign.total_distributed = campaign.campaign_performance
            .iter()
            .map(|p| p.calculated_earnings)
            .sum();

        emit!(FeesCalculated {
            campaign_id,
            total_distributed: campaign.total_distributed,
        });

        Ok(())
    }

    // Withdraw earnings for a provider
    pub fn withdraw_earnings(
        ctx: Context<WithdrawEarnings>,
        campaign_id: u32,
    ) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        let ad_provider = &mut ctx.accounts.ad_provider;

        // Find the provider's performance in this campaign
        let performance = campaign
            .campaign_performance
            .iter()
            .find(|p| p.provider == ad_provider.authority)
            .ok_or(ErrorCode::ProviderNotInCampaign)?;

        require!(
            performance.calculated_earnings > 0,
            ErrorCode::NoEarningsToWithdraw
        );

        let earnings = performance.calculated_earnings;

        // Transfer earnings from campaign to provider
        **campaign.to_account_info().try_borrow_mut_lamports()? -= earnings;
        **ad_provider.to_account_info().try_borrow_mut_lamports()? += earnings;

        // Update provider's total earnings
        ad_provider.total_earnings = ad_provider.total_earnings
            .checked_add(earnings)
            .ok_or(ErrorCode::CalculationError)?;

        emit!(EarningsWithdrawn {
            provider: ad_provider.authority,
            campaign_id,
            amount: earnings,
        });

        Ok(())
    }

    // Update provider information
    pub fn update_provider(
        ctx: Context<UpdateProvider>,
        name: Option<String>,
        location: Option<String>,
        contact_email: Option<String>,
        is_active: Option<bool>,
    ) -> Result<()> {
        let ad_provider = &mut ctx.accounts.ad_provider;
        let metadata = &mut ctx.accounts.provider_metadata;

        if let Some(name) = name {
            ad_provider.name = name.clone();
            metadata.name = name;
        }

        if let Some(location) = location {
            ad_provider.location = location.clone();
            metadata.location = location;
        }

        if let Some(email) = contact_email {
            ad_provider.contact_email = email;
        }

        if let Some(active) = is_active {
            ad_provider.is_active = active;
            metadata.is_active = active;
        }

        Ok(())
    }

    // Complete a campaign (mark as completed for fee calculation)
    pub fn complete_campaign(ctx: Context<CompleteCampaign>, campaign_id: u32) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        
        require!(
            campaign.campaign_status == CampaignStatus::Active,
            ErrorCode::CampaignNotActive
        );

        campaign.campaign_status = CampaignStatus::Completed;

        emit!(CampaignCompleted {
            campaign_id,
            authority: ctx.accounts.authority.key(),
        });

        Ok(())
    }

    // Get all providers from registry (view function)
    pub fn get_all_providers(ctx: Context<QueryProviders>) -> Result<Vec<Pubkey>> {
        let registry = &ctx.accounts.provider_registry;
        Ok(registry.providers.clone())
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
    #[msg("Provider registry not initialized")]
    RegistryNotInitialized,
    #[msg("Provider not found in registry")]
    ProviderNotInRegistry,
    #[msg("Registry is full, cannot add more providers")]
    RegistryFull,
    #[msg("Calculation error")]
    CalculationError,
    #[msg("Insufficient budget")]
    InsufficientBudget,
    #[msg("No views recorded")]
    NoViews,
    #[msg("Campaign not completed")]
    CampaignNotCompleted,
    #[msg("Campaign not active")]
    CampaignNotActive,
    #[msg("Provider not in campaign")]
    ProviderNotInCampaign,
    #[msg("No earnings to withdraw")]
    NoEarningsToWithdraw,
}