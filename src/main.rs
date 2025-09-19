// rcli csv -i input.csv -o output.json --header --d ','

use std::{fs, path::PathBuf};
use zxcvbn::zxcvbn;

use anyhow::Result;
use clap::Parser;
use rcli::{
    Base64SubCommand, Opts, SubCommand, TextSignFormat, TextSubCommand, process_csv,
    process_decode, process_encode, process_genpass, process_text_decrypt, process_text_encrypt,
    process_text_generate, process_text_sign, process_text_verify,
};

fn main() -> Result<()> {
    let opts = Opts::parse();
    match opts.cmd {
        SubCommand::Csv(opt) => {
            let output = if let Some(output) = opt.output {
                output.clone()
            } else {
                PathBuf::from(format!("output.{}", opt.format))
            };
            process_csv(opt.input, output, opt.format)?;
        }

        SubCommand::GenPass(opt) => {
            let password = process_genpass(
                opt.length,
                opt.no_uppercase,
                opt.no_lowercase,
                opt.no_number,
                opt.no_symbol,
            )?;
            println!("{}", password);

            let estimate = zxcvbn(&password, &[]);
            eprintln!("Password strenngth: {}", estimate.score());
        }

        SubCommand::Base64(subcmd) => match subcmd {
            Base64SubCommand::Encode(opt) => {
                let encoded = process_encode(opt.input, opt.format)?;
                println!("{}", encoded);
            }
            Base64SubCommand::Decode(opt) => {
                let decoded = process_decode(opt.input, opt.format)?;
                let decoded = String::from_utf8(decoded)?;
                println!("{}", decoded);
            }
        },

        SubCommand::Text(subcmd) => match subcmd {
            TextSubCommand::Sign(opt) => {
                let signed = process_text_sign(opt.input, opt.key, opt.format)?;
                println!("{}", signed);
            }

            TextSubCommand::Verify(opt) => {
                let ret = process_text_verify(opt.input, opt.key, opt.format, opt.sign)?;
                println!("{}", ret);
            }

            TextSubCommand::Generate(opt) => {
                let key = process_text_generate(opt.format)?;
                match opt.format {
                    TextSignFormat::Blake3 => {
                        let name = opt.output.join("blake3.txt");
                        fs::write(name, &key[0])?;
                    }
                    TextSignFormat::Ed25519 => {
                        let name = &opt.output;
                        fs::write(name.join("ed25519.sk"), &key[0])?;
                        fs::write(name.join("ed25519.pk"), &key[1])?;
                    }
                    TextSignFormat::ChaCha20Poly1305 => {
                        let name = &opt.output;
                        fs::write(name.join("chacha20poly1305.key"), &key[0])?;
                        fs::write(name.join("chacha20poly1305.nonce"), &key[1])?;
                    }
                }
            }

            TextSubCommand::Encrypt(opt) => {
                let encrypt = process_text_encrypt(opt.input, opt.key, opt.nonce)?;
                println!("{}", encrypt);
            }

            TextSubCommand::Decrypt(opt) => {
                let decrypt = process_text_decrypt(opt.input, opt.key, opt.nonce)?;
                println!("{}", decrypt);
            }
        },
    }
    Ok(())
}
