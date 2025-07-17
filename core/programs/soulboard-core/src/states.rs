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
    #[max_len(20)]
    pub devices: Vec<Soulboard>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Eq)]
pub enum CampaignStatus {
    Active,
    Paused,
    Completed,
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
    pub campaign_budget: u64, //in usd
    pub campaign_status: CampaignStatus,
    #[max_len(20)]
    pub campaign_providers: Vec<Pubkey>,
    #[max_len(20)]
    pub campaign_locations: Vec<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Eq)]
pub enum OrderStatus {
    Ordered,
    Delivered,
    Cancelled,
}

//If a  device order is made an order account is created
//Unless the order is cancelled or delivered

#[account]
#[derive(InitSpace)]
pub struct DeviceOrder {
    pub authority: Pubkey,
    pub device_id: u32,
    pub order_status: OrderStatus,
}

#[event]
pub struct DeviceOrdered {
    pub device_id: u32,
    pub device_state: DeviceState,
}

#[event]
pub struct ProviderRegistered {
    pub authority: Pubkey,
}

#[event]
pub struct CampaignCreated {
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
