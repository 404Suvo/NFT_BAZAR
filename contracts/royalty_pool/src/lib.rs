#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env, Vec};

#[contract]
pub struct RoyaltyPool;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Initialized,
    Token,
    RecipientLen,
    Recipient(u32),
    ShareBps(u32),
    Claimable(Address),
}

#[contractimpl]
impl RoyaltyPool {
    pub fn initialize(env: Env, admin: Address, token: Address, wallets: Vec<Address>, shares: Vec<i128>) {
        if env
            .storage()
            .instance()
            .has(&DataKey::Initialized)
        {
            panic!("already initialized");
        }
        if wallets.len() == 0 || wallets.len() != shares.len() {
            panic!("wallets/shares mismatch");
        }

        admin.require_auth();

        let mut total = 0_i128;
        let mut i = 0_u32;
        while i < wallets.len() {
            let wallet = wallets.get(i).unwrap();
            let share = shares.get(i).unwrap();
            if share <= 0 {
                panic!("share must be positive");
            }
            total += share;
            env.storage().instance().set(&DataKey::Recipient(i), &wallet);
            env.storage().instance().set(&DataKey::ShareBps(i), &share);
            i += 1;
        }
        if total != 10_000 {
            panic!("shares must sum to 10000 bps");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage()
            .instance()
            .set(&DataKey::RecipientLen, &wallets.len());
        env.storage().instance().set(&DataKey::Initialized, &true);
    }

    pub fn distribute(env: Env, from: Address, amount: i128) {
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let token_id: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_id);
        token_client.transfer(&from, &env.current_contract_address(), &amount);

        let len: u32 = env.storage().instance().get(&DataKey::RecipientLen).unwrap();
        let mut remainder = amount;
        let mut i = 0_u32;
        while i < len {
            let recipient: Address = env.storage().instance().get(&DataKey::Recipient(i)).unwrap();
            let share: i128 = env.storage().instance().get(&DataKey::ShareBps(i)).unwrap();
            let split = if i + 1 == len {
                remainder
            } else {
                (amount * share) / 10_000
            };
            remainder -= split;

            let current: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Claimable(recipient.clone()))
                .unwrap_or(0);
            env.storage()
                .persistent()
                .set(&DataKey::Claimable(recipient.clone()), &(current + split));
            i += 1;
        }

        env.events().publish((symbol_short!("royalty"),), amount);
    }

    pub fn claim(env: Env, recipient: Address) -> i128 {
        recipient.require_auth();

        let amount: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Claimable(recipient.clone()))
            .unwrap_or(0);
        if amount <= 0 {
            panic!("nothing to claim");
        }

        env.storage()
            .persistent()
            .set(&DataKey::Claimable(recipient.clone()), &0_i128);

        let token_id: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_id);
        token_client.transfer(&env.current_contract_address(), &recipient, &amount);

        env.events()
            .publish((symbol_short!("claim"), recipient), amount);
        amount
    }

    pub fn claimable(env: Env, recipient: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Claimable(recipient))
            .unwrap_or(0)
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, vec, Env};

    #[test]
    fn initialization_accepts_a_complete_royalty_split() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RoyaltyPool, ());
        let client = RoyaltyPoolClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let token = Address::generate(&env);
        let creator = Address::generate(&env);
        let stakers = Address::generate(&env);
        let treasury = Address::generate(&env);

        client.initialize(
            &admin,
            &token,
            &vec![&env, creator.clone(), stakers, treasury],
            &vec![&env, 5_000_i128, 3_000_i128, 2_000_i128],
        );

        assert_eq!(client.claimable(&creator), 0);
    }

    #[test]
    #[should_panic(expected = "shares must sum to 10000 bps")]
    fn initialization_rejects_an_incomplete_royalty_split() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RoyaltyPool, ());
        let client = RoyaltyPoolClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let token = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.initialize(
            &admin,
            &token,
            &vec![&env, recipient],
            &vec![&env, 9_999_i128],
        );
    }
}
