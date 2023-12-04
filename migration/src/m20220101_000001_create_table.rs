use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create account_code table
        manager
            .create_table(
                Table::create()
                    .table(AccountCode::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AccountCode::Root)
                            .blob(BlobSize::Long)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AccountCode::Procedures)
                            .blob(BlobSize::Long)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AccountCode::Module)
                            .blob(BlobSize::Long)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create account_storage table
        manager
            .create_table(
                Table::create()
                    .table(AccountStorage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AccountStorage::Root)
                            .blob(BlobSize::Long)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AccountStorage::Slots)
                            .blob(BlobSize::Long)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create account_vaults table
        manager
            .create_table(
                Table::create()
                    .table(AccountVaults::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AccountVaults::Root)
                            .blob(BlobSize::Long)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AccountVaults::Assets)
                            .blob(BlobSize::Long)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create account_keys table
        manager
            .create_table(
                Table::create()
                    .table(AccountKeys::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AccountKeys::AccountId)
                            .big_unsigned()
                            .not_null()
                            .primary_key(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("account_keys_account_id_fk")
                            .from(AccountKeys::Table, AccountKeys::AccountId)
                            .to(Accounts::Table, Accounts::Id),
                    )
                    .col(
                        ColumnDef::new(AccountKeys::KeyPair)
                            .blob(BlobSize::Long)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create accounts table
        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Accounts::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Accounts::CodeRoot)
                            .blob(BlobSize::Long)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("accounts_code_root_fk")
                            .from(Accounts::Table, Accounts::CodeRoot)
                            .to(AccountCode::Table, AccountCode::Root),
                    )
                    .col(
                        ColumnDef::new(Accounts::StorageRoot)
                            .blob(BlobSize::Long)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("accounts_storage_root_fk")
                            .from(Accounts::Table, Accounts::StorageRoot)
                            .to(AccountStorage::Table, AccountStorage::Root),
                    )
                    .col(
                        ColumnDef::new(Accounts::VaultRoot)
                            .blob(BlobSize::Long)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("accounts_vault_root_fk")
                            .from(Accounts::Table, Accounts::VaultRoot)
                            .to(AccountVaults::Table, AccountVaults::Root),
                    )
                    .col(ColumnDef::new(Accounts::Nonce).big_integer().not_null())
                    .col(ColumnDef::new(Accounts::Committed).boolean().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AccountCode::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AccountCode {
    Table,
    Root,       // blob not null primary key
    Procedures, // blob not null
    Module,     // blob not null
}

#[derive(DeriveIden)]
enum AccountStorage {
    Table,
    Root,  // blob not null primary key
    Slots, // blob not null
}

#[derive(DeriveIden)]
enum AccountVaults {
    Table,
    Root,   // blob not null primary key
    Assets, // blob not null
}

#[derive(DeriveIden)]
enum AccountKeys {
    Table,
    AccountId, // unsigned big int not null primary key, references accounts(id)
    KeyPair,   // blob not null
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,          // unsigned big int not null primary key
    CodeRoot,    // blob not null, references account_code(root)
    StorageRoot, // blob not null, references account_storage(root)
    VaultRoot,   // blob not null, references account_vaults(root)
    Nonce,       // big int not null
    Committed,   // boolean not null
}
