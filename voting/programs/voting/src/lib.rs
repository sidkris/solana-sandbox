// on-chain voting

use anchor_lang::prelude::*;

declare_id!("E61j5HM3hNhxHMruCHRp1M2dbuvahvdjZ4X1My15A16A");

#[program]
pub mod voting {
    use super::*;

    pub fn init_poll(ctx: Context<InitPoll>, _poll_id: u64, start: u64, end: u64,
                     name: String, description: String) -> Result<()> {
        let poll = &mut ctx.accounts.poll_account;
        poll.poll_description = description;
        poll.poll_name = name;
        poll.poll_voting_start = start;
        poll.poll_voting_end = end;
        Ok(());
    }

    pub fn initialize_candidate(ctx: Context<InitializeCandidate>,
                               _poll_id: u64,
                               candidate: String) -> Result<()> {
        ctx.accounts.candidate_account.candidate_name = candidate;
        ctx.accounts.poll_account.poll_option_index += 1;
        Ok(())
    }


    pub fn vote(ctx: Context<Vote>) -> Result<()> {


        Ok(())
    }

}

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitPoll {
    #[account(mut)] // making the account mutable as the signer balance will change
    pub signer: Signer<'info>,

    #[account(init,
             payer = signer,
             space = 8 + PollAccount::INIT_SPACE,
             seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
             bump)] // bump mandatory when see is used
    pub poll_account: Account<'info,  PollAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitializeCandidate {
    #[account(mut)] // making the account mutable as the signer balance will change
    pub signer: Signer<'info>,

    #[account(mut, seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()], bump)]
    pub poll_account: Account<'info,  PollAccount>,


    #[account(init,
             payer = signer,
             space = 8 + PollAccount::INIT_SPACE,
             seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
             bump)] // bump mandatory when see is used
    pub candidate_account: Account<'info,  CandidateAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct Vote <'info>{
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()], bump)]
    pub poll_account: Account<'info,  PollAccount>,


    #[account(seeds = [poll_id.to_le_bytes().as_ref(), candidate.as_ref()], bump)]
    pub candidate_account: Account<'info,  CandidateAccount>,

}


#[account]
#[derive(InitSpace)] // calculates available space on chain in order to calculate rent
pub struct PollAccount {
    #[max_len(32)] // used whenever you use a String data type
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