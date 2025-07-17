use std::str::FromStr;

use miden_objects::{
    Felt as NativeFelt,
    account::{AccountId as NativeAccountId, NetworkId},
};
use wasm_bindgen::prelude::*;

use super::felt::Felt;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct AccountId(NativeAccountId);

#[wasm_bindgen]
impl AccountId {
    #[wasm_bindgen(js_name = "fromHex")]
    pub fn from_hex(hex: &str) -> AccountId {
        let native_account_id = NativeAccountId::from_hex(hex).unwrap();
        AccountId(native_account_id)
    }

    #[wasm_bindgen(js_name = "fromBech32")]
    pub fn from_bech32(bech32: &str) -> AccountId {
        let (_, native_account_id) = NativeAccountId::from_bech32(bech32).unwrap();
        AccountId(native_account_id)
    }

    #[wasm_bindgen(js_name = "isFaucet")]
    pub fn is_faucet(&self) -> bool {
        self.0.is_faucet()
    }

    #[wasm_bindgen(js_name = "isRegularAccount")]
    pub fn is_regular_account(&self) -> bool {
        self.0.is_regular_account()
    }

    #[wasm_bindgen(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    /// Will turn the Account ID into its bech32 string representation. To avoid a potential
    /// wrongful encoding, this function will expect only IDs for either mainnet ("mm"),
    /// testnet ("mtst") or devnet ("mdev"). To use a custom bech32 prefix, use
    /// `Self::to_bech_32_custom`.
    #[wasm_bindgen(js_name = "toBech32")]
    pub fn to_bech32(&self, network_id: &str) -> Result<String, String> {
        match NetworkId::from_str(network_id) {
            Ok(NetworkId::Custom(_)) => {
                Err("Expected network id for either mainnet, testnet or devnet".to_owned())
            },
            Ok(net_id) => Ok(self.0.to_bech32(net_id)),
            Err(err) => Err(format!("Given network id is not valid: {err}")),
        }
    }

    /// Turn this Account ID into its bech32 string representation. This method accepts a custom
    /// network ID.
    #[wasm_bindgen(js_name = "toBech32Custom")]
    pub fn to_bech32_custom(&self, network_id: &str) -> Result<String, String> {
        let network_id = NetworkId::from_str(network_id)
            .map_err(|err| format!("Given network id is not valid: {err}"))?;
        match network_id {
            NetworkId::Custom(_) => Ok(self.0.to_bech32(network_id)),
            _ => Err("Expected a custom network id".to_owned()),
        }
    }

    pub fn prefix(&self) -> Felt {
        let native_felt: NativeFelt = self.0.prefix().as_felt();
        native_felt.into()
    }

    pub fn suffix(&self) -> Felt {
        let native_felt: NativeFelt = self.0.suffix();
        native_felt.into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAccountId> for AccountId {
    fn from(native_account_id: NativeAccountId) -> Self {
        AccountId(native_account_id)
    }
}

impl From<&NativeAccountId> for AccountId {
    fn from(native_account_id: &NativeAccountId) -> Self {
        AccountId(*native_account_id)
    }
}

impl From<AccountId> for NativeAccountId {
    fn from(account_id: AccountId) -> Self {
        account_id.0
    }
}

impl From<&AccountId> for NativeAccountId {
    fn from(account_id: &AccountId) -> Self {
        account_id.0
    }
}
