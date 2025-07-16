use anchor_lang::prelude::*;

declare_id!("Gt2gt87crJPt9Y3FbnVkAfdwxQ9cVVTE8mhAH6PKVHm7");

#[program]
pub mod core {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
