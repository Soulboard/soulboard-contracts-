use anchor_lang::prelude::*;

declare_id!("xF4A8Ksy6WSzJpskfiVUit4osedmBorP3bgDe9uKu2e");

pub const DEVICE_FEED_SEED: &[u8] = b"device_feed";

#[program]
pub mod oracle {
    use super::*;

    /// Create the feed once (e.g. when a device is first registered).
    pub fn initialize_device_feed(
        ctx: Context<InitializeDeviceFeed>,
        channel_id: u32,
        bump: u8,
    ) -> Result<()> {
        let feed = &mut ctx.accounts.feed;
        feed.channel_id = channel_id;
        feed.bump = bump;
        feed.last_entry_id = 0;
        feed.total_views = 0;
        feed.total_taps = 0;
        feed.last_update_ts = 0;
        feed.authority = *ctx.accounts.authority.key;

        emit!(DeviceFeedInitialized {
            // NEW
            channel_id,                       // NEW
            authority: feed.authority,        // NEW
            ts: Clock::get()?.unix_timestamp, // NEW
        });
        Ok(())
    }

    /// Keeper pushes deltas since `last_entry_id`. We store **running totals**.
    pub fn update_device_feed(
        ctx: Context<UpdateDeviceFeed>,
        _channel_id: u32,
        newest_entry_id: u32,
        delta_views: u64,
        delta_taps: u64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let feed = &mut ctx.accounts.feed;

        require!(newest_entry_id > feed.last_entry_id, OracleErr::NoNewData);
        require_keys_eq!(
            ctx.accounts.signer.key(),
            feed.authority,
            OracleErr::BadAuthority
        );

        feed.total_views = feed
            .total_views
            .checked_add(delta_views)
            .ok_or(OracleErr::Overflow)?;
        feed.total_taps = feed
            .total_taps
            .checked_add(delta_taps)
            .ok_or(OracleErr::Overflow)?;

        feed.last_entry_id = newest_entry_id;
        feed.last_update_ts = clock.unix_timestamp;
        emit!(DeviceFeedUpdated {
            // NEW
            channel_id: feed.channel_id,   // NEW
            new_entry_id: newest_entry_id, // NEW
            delta_views,                   // NEW
            delta_taps,                    // NEW
            total_views: feed.total_views, // NEW
            total_taps: feed.total_taps,   // NEW
            ts: clock.unix_timestamp,      // NEW
        });
        Ok(())
    }
}

/* --------------------------- Accounts ----------------------------------- */

#[account]
pub struct DeviceFeed {
    pub channel_id: u32,
    pub last_entry_id: u32,
    pub total_views: u64,
    pub total_taps: u64,
    pub last_update_ts: i64,
    pub authority: Pubkey,
    pub bump: u8,
}

#[derive(Accounts)]
#[instruction(channel_id: u32, bump: u8)]
pub struct InitializeDeviceFeed<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<DeviceFeed>(),
        seeds = [DEVICE_FEED_SEED, &channel_id.to_le_bytes()],
        bump,
    )]
    pub feed: Account<'info, DeviceFeed>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Device owner or DAO multisig that’s allowed to push updates
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(channel_id: u32)]
pub struct UpdateDeviceFeed<'info> {
    #[account(
        mut,
        seeds = [DEVICE_FEED_SEED, &channel_id.to_le_bytes()],
        bump = feed.bump,
    )]
    pub feed: Account<'info, DeviceFeed>,
    pub signer: Signer<'info>, // must match feed.authority
}

/* ----------------------------- Errors ----------------------------------- */
#[error_code]
pub enum OracleErr {
    #[msg("Caller not authorised")]
    BadAuthority,
    #[msg("Nothing new to record")]
    NoNewData,
    #[msg("Math overflow")]
    Overflow,
}

#[event] // NEW
pub struct DeviceFeedInitialized {
    // NEW
    pub channel_id: u32,   // NEW
    pub authority: Pubkey, // NEW
    pub ts: i64,           // NEW
}

#[event] // NEW
pub struct DeviceFeedUpdated {
    // NEW
    pub channel_id: u32,   // NEW
    pub new_entry_id: u32, // NEW
    pub delta_views: u64,  // NEW
    pub delta_taps: u64,   // NEW
    pub total_views: u64,  // NEW
    pub total_taps: u64,   // NEW
    pub ts: i64,           // NEW
}
