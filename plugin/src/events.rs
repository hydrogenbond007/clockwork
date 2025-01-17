use {
    anchor_lang::Discriminator,
    bincode::deserialize,
    clockwork_client::{thread::state::Thread, webhook::state::Request},
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        GeyserPluginError, ReplicaAccountInfo,
    },
    solana_program::{clock::Clock, pubkey::Pubkey, sysvar},
};

pub enum AccountUpdateEvent {
    Clock { clock: Clock },
    HttpRequest { request: Request },
    Thread { thread: Thread },
}

impl TryFrom<ReplicaAccountInfo<'_>> for AccountUpdateEvent {
    type Error = GeyserPluginError;
    fn try_from(account_info: ReplicaAccountInfo) -> Result<Self, Self::Error> {
        let account_pubkey = Pubkey::new(account_info.pubkey);
        let owner_pubkey = Pubkey::new(account_info.owner);

        // If the account is the sysvar clock, parse it.
        if account_pubkey.eq(&sysvar::clock::ID) {
            return Ok(AccountUpdateEvent::Clock {
                clock: deserialize::<Clock>(account_info.data).map_err(|_e| {
                    GeyserPluginError::AccountsUpdateError {
                        msg: "Failed to parsed sysvar clock account".into(),
                    }
                })?,
            });
        }

        // If the account belongs to the thread program, parse it.
        if owner_pubkey.eq(&clockwork_client::thread::ID) && account_info.data.len() > 8 {
            let d = &account_info.data[..8];
            if d.eq(&Thread::discriminator()) {
                return Ok(AccountUpdateEvent::Thread {
                    thread: Thread::try_from(account_info.data.to_vec()).map_err(|_| {
                        GeyserPluginError::AccountsUpdateError {
                            msg: "Failed to parse Clockwork thread account".into(),
                        }
                    })?,
                });
            }
        }

        // If the account belongs to the webhook program, parse in
        if owner_pubkey.eq(&clockwork_client::webhook::ID) && account_info.data.len() > 8 {
            return Ok(AccountUpdateEvent::HttpRequest {
                request: Request::try_from(account_info.data.to_vec()).map_err(|_| {
                    GeyserPluginError::AccountsUpdateError {
                        msg: "Failed to parse Clockwork http request".into(),
                    }
                })?,
            });
        }

        Err(GeyserPluginError::AccountsUpdateError {
            msg: "Account is not relevant to Clockwork plugin".into(),
        })
    }
}
