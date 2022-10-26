use {
    crate::{errors::*, objects::*},
    anchor_lang::prelude::*,
    clockwork_network_program::objects::{Worker, WorkerAccount},
};

/// Accounts required by the `thread_kickoff` instruction.
#[derive(Accounts)]
#[instruction(data_hash: Option<u64>)]
pub struct ThreadKickoff<'info> {
    /// The signatory.
    #[account(mut)]
    pub signatory: Signer<'info>,

    /// The thread to kickoff.
    #[account(
        mut,
        seeds = [
            SEED_THREAD,
            thread.authority.as_ref(),
            thread.id.as_bytes(),
        ],
        bump,
        constraint = !thread.paused @ ClockworkError::ThreadPaused,
        constraint = thread.next_instruction.is_none() @ ClockworkError::ThreadBusy,
    )]
    pub thread: Box<Account<'info, Thread>>,

    /// The worker.
    #[account(
        address = worker.pubkey(),
        has_one = signatory
    )]
    pub worker: Account<'info, Worker>,
}

pub fn handler(ctx: Context<ThreadKickoff>, data_hash: Option<u64>) -> Result<()> {
    // Get accounts.
    let thread = &mut ctx.accounts.thread;

    // If this thread does not have a next_instruction, verify the thread's trigger condition is active.
    thread.kickoff(data_hash, ctx.remaining_accounts)?;

    Ok(())
}