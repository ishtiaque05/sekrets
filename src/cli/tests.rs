use googletest::prelude::*;

use crate::cli::build_cli;

#[googletest::test]
fn test_cli_encrypt_parsing() {
    let cli = build_cli();
    let matches = cli.get_matches_from(vec!["sekret", "encrypt", "-f", "../fixtures/foo.txt"]);

    let (subcommand_name, sub_matches) = matches.subcommand().expect("Expected a subcommand");
    expect_that!(subcommand_name, eq("encrypt"));

    let file = sub_matches.get_one::<String>("file").expect("File not found");
    expect_that!(file, eq("../fixtures/foo.txt"));
}

#[googletest::test]
fn test_missing_file_encrypt() {
    let cli = build_cli();

    let result = cli.try_get_matches_from(vec!["sekrets", "encrypt"]);

    expect_pred!(result.is_err());
    expect_that!(
        result.unwrap_err().to_string(), 
        contains_substring("the following required arguments were not provided:\n  --file <FILE>")
    );

}

#[googletest::test]
fn test_cli_decrypt_parsing() {
    let cli = build_cli();
    let matches = cli.get_matches_from(vec![
        "sekrets",
        "decrypt",
        "-f",
        "test.txt",
        "-a",
        "github",
        "-a",
        "bank",
    ]);

    let (subcommand_name, sub_matches) = matches.subcommand().expect("Expected a subcommand");
    expect_that!(subcommand_name, eq("decrypt"));

    let file = sub_matches.get_one::<String>("file").expect("File not found");
    expect_that!(file, eq("test.txt"));

    let accounts: Vec<_> = sub_matches
        .get_many::<String>("accounts")
        .expect("Accounts not found")
        .collect();
    
    expect_that!(accounts.len(), eq(2));
    expect_that!(accounts[0], eq("github"));
    expect_that!(accounts[1], eq("bank"));
}

#[googletest::test]
fn test_decrypt_missing_args() {
    let cli = build_cli();
    let result = cli.try_get_matches_from(vec!["sekrets", "decrypt", "-f", "test.txt"]);

    expect_pred!(result.is_err());
    expect_that!(
        result.unwrap_err().to_string(), 
        contains_substring("the following required arguments were not provided:\n  --accounts")
    );
}