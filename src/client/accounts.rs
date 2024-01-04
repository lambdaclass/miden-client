use super::Client;
use crypto::Felt;
use miden_lib::{accounts::{faucets, wallets}, AuthScheme};
use objects::{
    accounts::{Account, AccountId, AccountStorage, AccountStub, AccountType},
    assembly::ModuleAst,
    assets::{Asset, TokenSymbol},
    Digest,
};
use rand::{rngs::ThreadRng, Rng};

use crate::{errors::ClientError, store::accounts::AuthInfo};

pub enum AccountTemplate {
    BasicWallet {
        mutable_code: bool,
        storage_mode: AccountStorageMode,
    },
    FungibleFaucet {
        token_symbol: TokenSymbol,
        decimals: u8,
        max_supply: u64,
        storage_mode: AccountStorageMode,
    },
}

pub enum AccountStorageMode {
    Local,
    OnChain,
}

impl Client {
    // ACCOUNT INSERTION
    // --------------------------------------------------------------------------------------------

    // ACCOUNT CREATION
    // --------------------------------------------------------------------------------------------

    pub fn new_account(&mut self, template: AccountTemplate) -> Result<Account, ClientError> {
        let mut rng = rand::thread_rng();

        let account = match template {
            AccountTemplate::BasicWallet {
                mutable_code,
                storage_mode,
            } => self.new_basic_wallet(mutable_code, &mut rng, storage_mode),
            AccountTemplate::FungibleFaucet {
                token_symbol,
                decimals,
                max_supply,
                storage_mode,
            } => {
                self.new_fungible_faucet(token_symbol, decimals, max_supply, &mut rng, storage_mode)
            }
        }?;

        Ok(account)
    }

    fn new_basic_wallet(
        &mut self,
        mutable_code: bool,
        rng: &mut ThreadRng,
        account_storage_mode: AccountStorageMode,
    ) -> Result<Account, ClientError> {
        if let AccountStorageMode::OnChain = account_storage_mode {
            todo!("Recording the account on chain is not supported yet");
        }

        let key_pair: objects::crypto::dsa::rpo_falcon512::KeyPair =
            objects::crypto::dsa::rpo_falcon512::KeyPair::new().map_err(ClientError::AuthError)?;

        let auth_scheme: AuthScheme = AuthScheme::RpoFalcon512 {
            pub_key: key_pair.public_key(),
        };

        // we need to use an initial seed to create the wallet account
        let init_seed: [u8; 32] = rng.gen();

        let (account, _seed) = if !mutable_code {
            wallets::create_basic_wallet(
                init_seed,
                auth_scheme,
                AccountType::RegularAccountImmutableCode,
            )
        } else {
            wallets::create_basic_wallet(
                init_seed,
                auth_scheme,
                AccountType::RegularAccountUpdatableCode,
            )
        }
        .map_err(ClientError::AccountError)?;

        self.insert_account(&account, &AuthInfo::RpoFalcon512(key_pair))?;
        Ok(account)
    }

    fn new_fungible_faucet(
        &mut self,
        token_symbol: TokenSymbol,
        decimals: u8,
        max_supply: u64,
        rng: &mut ThreadRng,
        account_storage_mode: AccountStorageMode,
    ) -> Result<Account, ClientError> {
        if let AccountStorageMode::OnChain = account_storage_mode {
            todo!("On-chain accounts are not supported yet");
        }

        let key_pair: objects::crypto::dsa::rpo_falcon512::KeyPair =
            objects::crypto::dsa::rpo_falcon512::KeyPair::new().map_err(ClientError::AuthError)?;

        let auth_scheme: AuthScheme = AuthScheme::RpoFalcon512 {
            pub_key: key_pair.public_key(),
        };

        // we need to use an initial seed to create the wallet account
        let init_seed: [u8; 32] = rng.gen();

        let (account, _seed) = faucets::create_basic_fungible_faucet(
            init_seed,
            token_symbol,
            decimals,
            Felt::try_from(max_supply.to_le_bytes().as_slice())
                .expect("u64 can be safely converted to a field element"),
            auth_scheme,
        )
        .map_err(ClientError::AccountError)?;

        self.insert_account(&account, &AuthInfo::RpoFalcon512(key_pair))?;
        Ok(account)
    }

    /// Inserts a new account into the client's store.
    pub fn insert_account(
        &mut self,
        account: &Account,
        auth_info: &AuthInfo,
    ) -> Result<(), ClientError> {
        self.store
            .insert_account(account, auth_info)
            .map_err(ClientError::StoreError)
    }

    // ACCOUNT DATA RETRIEVAL
    // --------------------------------------------------------------------------------------------

    /// Returns summary info about the accounts managed by this client.
    ///
    /// TODO: replace `AccountStub` with a more relevant structure.
    pub fn get_accounts(&self) -> Result<Vec<AccountStub>, ClientError> {
        self.store.get_accounts().map_err(|err| err.into())
    }

    /// Returns summary info about the specified account.
    pub fn get_account_by_id(&self, account_id: AccountId) -> Result<AccountStub, ClientError> {
        self.store
            .get_account_by_id(account_id)
            .map_err(|err| err.into())
    }

    /// Returns key pair structure for an Account Id.
    pub fn get_account_auth(&self, account_id: AccountId) -> Result<AuthInfo, ClientError> {
        self.store
            .get_account_auth(account_id)
            .map_err(|err| err.into())
    }

    /// Returns vault assets from a vault root.
    pub fn get_vault_assets(&self, vault_root: Digest) -> Result<Vec<Asset>, ClientError> {
        self.store
            .get_vault_assets(vault_root)
            .map_err(|err| err.into())
    }

    /// Returns account code data from a root.
    pub fn get_account_code(
        &self,
        code_root: Digest,
    ) -> Result<(Vec<Digest>, ModuleAst), ClientError> {
        self.store
            .get_account_code(code_root)
            .map_err(|err| err.into())
    }

    /// Returns account storage data from a storage root.
    pub fn get_account_storage(&self, storage_root: Digest) -> Result<AccountStorage, ClientError> {
        self.store
            .get_account_storage(storage_root)
            .map_err(|err| err.into())
    }

    /// Returns historical states for the account with the specified ID.
    ///
    /// TODO: wrap `Account` in a type with additional info.
    /// TODO: consider changing the interface to support pagination.
    pub fn get_account_history(&self, _account_id: AccountId) -> Result<Vec<Account>, ClientError> {
        todo!()
    }

    /// Returns detailed information about the current state of the account with the specified ID.
    ///
    /// TODO: wrap `Account` in a type with additional info (e.g., status).
    /// TODO: consider adding `nonce` as another parameter to identify a specific account state.
    pub fn get_account_details(&self, _account_id: AccountId) -> Result<Account, ClientError> {
        todo!()
    }
}
