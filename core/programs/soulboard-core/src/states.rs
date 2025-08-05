use anchor_lang::prelude::*;

//Keeps track of all the devices in the soulboard system
//Device id is the channel id of the device
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct Soulboard {
    pub device_id: u32,            //Channel id of the device
    pub device_state: DeviceState, //State of the device
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Eq)]
pub enum DeviceState {
    Available,
    Booked,
    Ordered,
    Paused,
}

//AdProvider is the account that manages out the devices and plant them
// at the locations
#[account]
#[derive(InitSpace)]
pub struct AdProvider {
    pub authority: Pubkey,
    #[max_len(10)] // Reduced from 20 to 10 devices
    pub devices: Vec<Soulboard>,
    #[max_len(32)] // Reduced from 50 to 32
    pub name: String, // Provider name
    #[max_len(64)] // Reduced from 100 to 64
    pub location: String, // Provider location
    #[max_len(32)] // Reduced from 50 to 32
    pub contact_email: String, // Contact information
    pub rating: u8,           // Rating out of 100
    pub total_campaigns: u32, // Total campaigns served
    pub is_active: bool,      // Active status
    pub total_earnings: u64,  // Total earnings in lamports
    pub pending_payments: u64, // Pending payments to be withdrawn
}

// Global registry to track all providers
#[account]
#[derive(InitSpace)]
pub struct ProviderRegistry {
    pub total_providers: u32,
    #[max_len(50)] // Reduced from 1000 to 50 to stay under size limits
    pub providers: Vec<Pubkey>, // List of all provider authorities
}

// Provider metadata for easier querying
#[account]
#[derive(InitSpace)]
pub struct ProviderMetadata {
    pub authority: Pubkey,
    pub provider_pda: Pubkey,
    #[max_len(32)] // Reduced to match AdProvider
    pub name: String,
    #[max_len(64)] // Reduced to match AdProvider
    pub location: String,
    pub device_count: u32,
    pub available_devices: u32,
    pub rating: u8,
    pub is_active: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Eq)]
pub enum CampaignStatus {
    Active,
    Paused,
    Completed,
}

// Provider performance tracking within a campaign
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct ProviderPerformance {
    pub provider: Pubkey,           // Provider authority
    pub device_id: u32,            // Device being used
    pub total_views: u64,          // Total views from oracle
    pub total_taps: u64,           // Total taps from oracle
    pub calculated_earnings: u64,   // Final calculated earnings
    pub base_fee_earned: u64,      // Base fee portion
    pub performance_fee_earned: u64, // Performance-based portion
}

#[account]
#[derive(InitSpace)]
pub struct Campaign {
    pub authority: Pubkey,
    pub campaign_id: u32,
    #[max_len(20)]
    pub campaign_name: String,
    #[max_len(100)]
    pub campaign_description: String,
    pub campaign_budget: u64, //in lamports
    pub campaign_status: CampaignStatus,
    #[max_len(20)]
    pub campaign_providers: Vec<Pubkey>,
    #[max_len(20)]
    pub campaign_locations: Vec<Pubkey>,
    
    // Fee calculation fields
    pub running_days: u32,         // Total running days
    pub hours_per_day: u32,        // Hours per day
    pub base_fee_per_hour: u64,    // Base fee per hour in lamports
    pub platform_fee: u64,         // Platform fee (2% of budget)
    pub total_distributed: u64,    // Total amount distributed to providers
    
    // Performance tracking
    #[max_len(20)]
    pub campaign_performance: Vec<ProviderPerformance>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Eq)]
pub enum OrderStatus {
    Ordered,
    Delivered,
    Cancelled,
}

//If a device order is made an order account is created
//Unless the order is cancelled or delivered
#[account]
#[derive(InitSpace)]
pub struct DeviceOrder {
    pub authority: Pubkey,
    pub device_id: u32,
    pub order_status: OrderStatus,
}

// Events
#[event]
pub struct DeviceOrdered {
    pub device_id: u32,
    pub device_state: DeviceState,
}

#[event]
pub struct ProviderRegistered {
    pub authority: Pubkey,
    pub name: String,
    pub location: String,
}

#[event]
pub struct CampaignCreated {
    pub authority: Pubkey,
}

#[event]
pub struct CampaignCompleted {
    pub campaign_id: u32,
    pub authority: Pubkey,
}

#[event]
pub struct BudgetAdded {
    pub authority: Pubkey,
}

#[event]
pub struct LocationAdded {
    pub campaign_id: u32,
    pub location: Pubkey,
}

#[event]
pub struct LocationRemoved {
    pub campaign_id: u32,
    pub location: Pubkey,
}

#[event]
pub struct RegistryInitialized {
    pub registry: Pubkey,
}

#[event]
pub struct ProviderMetadataUpdated {
    pub authority: Pubkey,
    pub available_devices: u32,
}

#[event]
pub struct PerformanceUpdated {
    pub campaign_id: u32,
    pub device_id: u32,
    pub total_views: u64,
    pub total_taps: u64,
}

#[event]
pub struct FeesCalculated {
    pub campaign_id: u32,
    pub total_distributed: u64,
}

#[event]
pub struct EarningsWithdrawn {
    pub provider: Pubkey,
    pub campaign_id: u32,
    pub amount: u64,
}