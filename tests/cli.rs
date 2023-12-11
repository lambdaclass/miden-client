use assert_cmd::prelude::*; // Add methods on commands
use escargot::CargoBuild;
use predicates::prelude::*; // Used for writing assertions

#[test]
fn test_cli_create_account_works() -> Result<(), Box<dyn std::error::Error>> {
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
fn test_cli_list_accounts() -> Result<(), Box<dyn std::error::Error>> {
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
