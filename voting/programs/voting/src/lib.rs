// on-chain voting

use anchor_lang::prelude::*;

declare_id!("E61j5HM3hNhxHMruCHRp1M2dbuvahvdjZ4X1My15A16A");

#[program]
pub mod voting {
    use super::*;

    pub fn init_poll(
        ctx: Context<InitPoll>,
        _poll_id: u64,
        start: u64,
        end: u64,
        name: String,
        description: String,
    ) -> Result<()> {
        let poll = &mut ctx.accounts.poll_account;

        poll.poll_description = description;
        poll.poll_name = name;
        poll.poll_voting_start = start;
        poll.poll_voting_end = end;
        poll.poll_option_index = 0;

        Ok(())
    }

    pub fn initialize_candidate(
        ctx: Context<InitializeCandidate>,
        _poll_id: u64,
        candidate: String,
    ) -> Result<()> {
        ctx.accounts.candidate_account.candidate_name = candidate;
        ctx.accounts.candidate_account.candidate_votes = 0;

        ctx.accounts.poll_account.poll_option_index += 1;

        Ok(())
    }

    pub fn vote(
        ctx: Context<Vote>,
        _poll_id: u64,
        _candidate: String,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp as u64;

        let poll = &ctx.accounts.poll_account;

        if current_time > poll.poll_voting_end {
            return Err(ErrorCode::VotingEnded.into());
        }

        if current_time < poll.poll_voting_start {
            return Err(ErrorCode::VotingNotStarted.into());
        }

        ctx.accounts.candidate_account.candidate_votes += 1;

        Ok(())
    }
}


// --------------------------------------------------
// Init Poll
// --------------------------------------------------

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitPoll<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = 8 + PollAccount::INIT_SPACE,
        seeds = [
            b"poll",
            poll_id.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub poll_account: Account<'info, PollAccount>,

    pub system_program: Program<'info, System>,
}


// --------------------------------------------------
// Initialize Candidate
// --------------------------------------------------

#[derive(Accounts)]
#[instruction(poll_id: u64, candidate: String)]
pub struct InitializeCandidate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            b"poll",
            poll_id.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub poll_account: Account<'info, PollAccount>,

    #[account(
        init,
        payer = signer,
        space = 8 + CandidateAccount::INIT_SPACE,
        seeds = [
            b"candidate",
            poll_id.to_le_bytes().as_ref(),
            candidate.as_bytes()
        ],
        bump
    )]
    pub candidate_account: Account<'info, CandidateAccount>,

    pub system_program: Program<'info, System>,
}


// --------------------------------------------------
// Vote
// --------------------------------------------------

#[derive(Accounts)]
#[instruction(poll_id: u64, candidate: String)]
pub struct Vote<'info> {
    pub signer: Signer<'info>,

    #[account(
        seeds = [
            b"poll",
            poll_id.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub poll_account: Account<'info, PollAccount>,

    #[account(
        mut,
        seeds = [
            b"candidate",
            poll_id.to_le_bytes().as_ref(),
            candidate.as_bytes()
        ],
        bump
    )]
    pub candidate_account: Account<'info, CandidateAccount>,
}


// --------------------------------------------------
// State
// --------------------------------------------------

#[account]
#[derive(InitSpace)]
pub struct PollAccount {
    #[max_len(32)]
    pub poll_name: String,

    #[max_len(250)]
    pub poll_description: String,

    pub poll_voting_start: u64,
    pub poll_voting_end: u64,
    pub poll_option_index: u64,
}


#[account]
#[derive(InitSpace)]
pub struct CandidateAccount {
    #[max_len(32)]
    pub candidate_name: String,

    pub candidate_votes: u64,
}


// --------------------------------------------------
// Errors
// --------------------------------------------------

#[error_code]
pub enum ErrorCode {
    #[msg("Voting has not started yet.")]
    VotingNotStarted,

    #[msg("Voting has ended.")]
    VotingEnded,
}