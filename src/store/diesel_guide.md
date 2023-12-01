# Diesel setup Instructions
[source](https://diesel.rs/guides/getting-started.html)

**Note:** this guide only needs to be followed in case `diesel_migrations` crate is not used, which is **not** the case for this project. This guide is only for reference on how to setup diesel with static sql files.

## Install libsqlite3:
  - Ubuntu: `sudo apt intsall libsqlite3-dev`
  - MacOS:  preinstalled in modern versions.

## Install Diesel CLI:
`cargo install diesel_cli --no-default-features --features sqlite`

## Database creation:
`diesel setup --database-url=store.sqlite3`

This will create a `store.sqlite3` file in the current root of the proyect.

## Modify `diesel.toml`:
  ```toml
    # For documentation on how to configure this file,
    # see https://diesel.rs/guides/configuring-diesel-cli

    [print_schema]
    file = "src/store/schema.rs"
    custom_type_derives = ["diesel::query_builder::QueryId"]

    [migrations_directory]
    dir = "src/store/migrations"
  ```

## Generate migration file:
 `diesel migration generate miden_client_store --database-url=store.sqlite3`
  
## Modify `src/store/migrations/TIMESTAMP_miden_client_store` file:

  ```sql
    -- Create account_code table
    CREATE TABLE account_code (
        root BLOB NOT NULL,         -- root of the Merkle tree for all exported procedures in account module.
        procedures BLOB NOT NULL,   -- serialized procedure digests for the account code.
        module BLOB NOT NULL,       -- serialized ModuleAst for the account code.
        PRIMARY KEY (root)
    );

    -- Create account_storage table
    CREATE TABLE account_storage (
        root BLOB NOT NULL,         -- root of the account storage Merkle tree.
        slots BLOB NOT NULL,        -- serialized key-value pair of non-empty account slots.
        PRIMARY KEY (root)
    );

    -- Create account_vaults table
    CREATE TABLE account_vaults (
        root BLOB NOT NULL,         -- root of the Merkle tree for the account vault.
        assets BLOB NOT NULL,       -- serialized account vault assets.
        PRIMARY KEY (root)
    );

    -- Create account_keys table
    CREATE TABLE account_keys (
        account_id UNSIGNED BIG INT NOT NULL, -- ID of the account
        key_pair BLOB NOT NULL,               -- key pair
        PRIMARY KEY (account_id),
        FOREIGN KEY (account_id) REFERENCES accounts(id)
    );

    -- Create accounts table
    CREATE TABLE accounts (
        id UNSIGNED BIG INT NOT NULL,  -- account ID.
        code_root BLOB NOT NULL,       -- root of the account_code Merkle tree.
        storage_root BLOB NOT NULL,    -- root of the account_storage Merkle tree.
        vault_root BLOB NOT NULL,      -- root of the account_vault Merkle tree.
        nonce BIGINT NOT NULL,         -- account nonce.
        committed BOOLEAN NOT NULL,    -- true if recorded, false if not.
        PRIMARY KEY (id),
        FOREIGN KEY (code_root) REFERENCES account_code(root),
        FOREIGN KEY (storage_root) REFERENCES account_storage(root),
        FOREIGN KEY (vault_root) REFERENCES account_vaults(root)
    );

    -- Create input notes table
    CREATE TABLE input_notes (
        hash BLOB NOT NULL,                                     -- the note hash
        nullifier BLOB NOT NULL,                                -- the nullifier of the note
        script BLOB NOT NULL,                                   -- the serialized NoteScript, including script hash and ProgramAst
        vault BLOB NOT NULL,                                    -- the serialized NoteVault, including vault hash and list of assets
        inputs BLOB NOT NULL,                                   -- the serialized NoteInputs, including inputs hash and list of inputs
        serial_num BLOB NOT NULL,                               -- the note serial number
        sender_id UNSIGNED BIG INT NOT NULL,                    -- the account ID of the sender
        tag UNSIGNED BIG INT NOT NULL,                          -- the note tag
        num_assets UNSIGNED BIG INT NOT NULL,                   -- the number of assets in the note
        inclusion_proof BLOB NOT NULL,                          -- the inclusion proof of the note against a block number
        recipients BLOB NOT NULL,                               -- a list of account IDs of accounts which can consume this note
        status TEXT CHECK( status IN ('pending', 'committed')), -- the status of the note - either pending or committed
        commit_height UNSIGNED BIG INT NOT NULL,                -- the block number at which the note was included into the chain
        PRIMARY KEY (hash)
    );
```

## Apply migration:
`diesel migration run --database-url=store.sqlite3`

## Notice
If you followed this guide, some parts of the code will need to be modified to mark the default database url as `store.sqlite`.