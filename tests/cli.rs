use assert_cmd::prelude::*; // Add methods on commands
use escargot::CargoBuild;
use predicates::prelude::*; // Used for writing assertions

// ACCOUNT
// ================================================================================================

#[test]
fn test_cli_account_new_works() -> Result<(), Box<dyn std::error::Error>> {
    // IMPORTANT: this test will fail if on multiple runs, because the test
    //            will try to create the same content twice (constraint violation)

    let run = CargoBuild::new()
        .bin(env!("CARGO_PKG_NAME"))
        .features("testing")
        .run()
        .unwrap()
        .command()
        .args([
            "account",
            "new",
            "fungible-faucet",
            "-t",
            "TEST",
            "-d",
            "10",
            "-m",
            "10000",
        ])
        .unwrap();

    run.assert().success().stdout(predicate::str::contains(
        "Succesfully created and stored Account ID:",
    ));

    Ok(())
}

#[test]
fn test_cli_account_list_accounts_works() -> Result<(), Box<dyn std::error::Error>> {
    let run = CargoBuild::new()
        .bin(env!("CARGO_PKG_NAME"))
        .features("testing")
        .run()
        .unwrap()
        .command()
        .args(["account", "list"])
        .unwrap();

    run.assert()
        .success()
        .stdout(predicate::str::contains("│ account id"));
    Ok(())
}

// INPUT-NOTES
// ================================================================================================

#[test]
fn test_cli_input_notes_list_works() -> Result<(), Box<dyn std::error::Error>> {
    let run = CargoBuild::new()
        .bin(env!("CARGO_PKG_NAME"))
        .features("testing")
        .run()
        .unwrap()
        .command()
        .args(["input-notes", "list"])
        .unwrap();

    run.assert()
        .success()
        .stdout(predicate::str::contains("│ hash"));
    Ok(())
}

#[test]
fn test_cli_input_notes_show_works() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: it would be nice to generate a input note before, retrieve it through
    //       the cli and try to show it

    let account_hash = format!("0x{}", "0".repeat(64));

    let run = CargoBuild::new()
        .bin(env!("CARGO_PKG_NAME"))
        .features("testing")
        .run()
        .unwrap()
        .command()
        .args(["input-notes", "show", &account_hash])
        .unwrap();

    let pattern = format!("input note with hash {} not found", account_hash);
    run.assert()
        .success()
        .stdout(predicate::str::contains(pattern));
    Ok(())
}

// SYNC-STATE
// ================================================================================================
#[test]
fn test_cli_sync_state_sync_state_works() -> Result<(), Box<dyn std::error::Error>> {
    // IMPORTANT: this test will fail if on multiple runs, because the
    //            mock data will try to create the same content twice (constraint violation)
    let _mock_run = CargoBuild::new()
        .bin(env!("CARGO_PKG_NAME"))
        .features("testing")
        .run()
        .unwrap()
        .command()
        .args(["mock-data"])
        .unwrap()
        .assert()
        .success();

    let run = CargoBuild::new()
        .bin(env!("CARGO_PKG_NAME"))
        .features("testing")
        .run()
        .unwrap()
        .command()
        .args(["sync-state", "-s"])
        .unwrap();

    run.assert()
        .success()
        .stdout(predicate::str::contains("state synced to block"));

    Ok(())
}

#[test]
fn test_cli_sync_state_add_tag_works() -> Result<(), Box<dyn std::error::Error>> {
    let run = CargoBuild::new()
        .bin(env!("CARGO_PKG_NAME"))
        .features("testing")
        .run()
        .unwrap()
        .command()
        .args(["sync-state", "-a", "518"])
        .unwrap();

    run.assert()
        .success()
        .stdout(predicate::str::contains("tag 518 added"));

    Ok(())
}

#[test]
fn test_cli_sync_state_list_tags_works() -> Result<(), Box<dyn std::error::Error>> {
    let run = CargoBuild::new()
        .bin(env!("CARGO_PKG_NAME"))
        .features("testing")
        .run()
        .unwrap()
        .command()
        .args(["sync-state", "-l"])
        .unwrap();

    run.assert()
        .success()
        .stdout(predicate::str::contains("tags:"));

    Ok(())
}

#[test]
fn test_cli_sync_state_block_number_works() -> Result<(), Box<dyn std::error::Error>> {
    let run = CargoBuild::new()
        .bin(env!("CARGO_PKG_NAME"))
        .features("testing")
        .run()
        .unwrap()
        .command()
        .args(["sync-state", "-b"])
        .unwrap();

    run.assert()
        .success()
        .stdout(predicate::str::contains("block number:"));

    Ok(())
}
