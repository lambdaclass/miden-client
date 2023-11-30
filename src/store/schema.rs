// @generated automatically by Diesel CLI.

diesel::table! {
    account_code (root) {
        root -> Binary,
        procedures -> Binary,
        module -> Binary,
    }
}

diesel::table! {
    account_keys (account_id) {
        account_id -> BigInt,
        key_pair -> Binary,
    }
}

diesel::table! {
    account_storage (root) {
        root -> Binary,
        slots -> Binary,
    }
}

diesel::table! {
    account_vaults (root) {
        root -> Binary,
        assets -> Binary,
    }
}

diesel::table! {
    accounts (id) {
        id -> BigInt,
        code_root -> Binary,
        storage_root -> Binary,
        vault_root -> Binary,
        nonce -> BigInt,
        committed -> Bool,
    }
}

diesel::table! {
    input_notes (hash) {
        hash -> Binary,
        nullifier -> Binary,
        script -> Binary,
        vault -> Binary,
        inputs -> Binary,
        serial_num -> Binary,
        sender_id -> BigInt,
        tag -> BigInt,
        num_assets -> BigInt,
        inclusion_proof -> Binary,
        recipients -> Binary,
        status -> Nullable<Text>,
        commit_height -> BigInt,
    }
}

diesel::joinable!(account_keys -> accounts (account_id));
diesel::joinable!(accounts -> account_code (code_root));
diesel::joinable!(accounts -> account_storage (storage_root));
diesel::joinable!(accounts -> account_vaults (vault_root));

diesel::allow_tables_to_appear_in_same_query!(
    account_code,
    account_keys,
    account_storage,
    account_vaults,
    accounts,
    input_notes,
);
