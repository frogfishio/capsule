// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};

use capsule_lib::{Capsule, Encoding, ParseOptions};
use capsule_lib::ascii_header::HeaderField;

mod spec;

#[derive(Parser)]
#[command(name = "capsule")]
#[command(about = "Capsule container tool", long_about = None)]
#[command(arg_required_else_help = true)]
struct Cli {
    /// Print license information and exit
    #[arg(long, global = true)]
    license: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a capsule from header fields and a payload file
    Pack {
        /// Optional capsule.toml spec file (flags override spec values)
        #[arg(long)]
        spec: Option<PathBuf>,

        /// Output capsule file
        #[arg(short, long)]
        out: Option<PathBuf>,

        /// Payload file path
        #[arg(short, long)]
        payload: Option<PathBuf>,

        /// Encoding: A (ASCII), B (Base64), C (CBOR)
        #[arg(short = 'e', long)]
        encoding: Option<String>,

        /// Header field (repeatable): key=value (ASCII encoding only)
        #[arg(long = "header", value_parser = parse_kv)]
        header: Vec<(String, String)>,

        /// Raw header block bytes from file (Base64/CBOR encodings)
        #[arg(long)]
        header_file: Option<PathBuf>,
    },

    /// Print capsule metadata and (for ASCII) header fields
    Info {
        /// Capsule file path
        file: PathBuf,

        /// Verify CRC and validate encoding
        #[arg(long)]
        verify: bool,
    },

    /// Verify capsule structure, encoding validity, and CRC
    Verify {
        /// Capsule file path
        file: PathBuf,
    },

    /// Decode and extract the payload (and optionally header) to files
    Unpack {
        /// Capsule file path
        file: PathBuf,

        /// Output file for decoded payload bytes
        #[arg(short, long)]
        out_payload: PathBuf,

        /// Output file for decoded header bytes
        #[arg(long)]
        out_header: Option<PathBuf>,

        /// Verify CRC and validate encoding before extracting
        #[arg(long)]
        verify: bool,

        /// Extract raw (encoded) header/payload bytes as stored (no decoding)
        #[arg(long)]
        raw: bool,
    },
}

fn parse_kv(s: &str) -> std::result::Result<(String, String), String> {
    let Some(eq) = s.find('=') else {
        return Err("expected key=value".to_string());
    };
    Ok((s[..eq].to_string(), s[eq + 1..].to_string()))
}

fn main() -> Result<()> {
    let semver = env!("CAPSULE_SEMVER");

    // Build clap command dynamically so --version uses our repo-derived version.
    let mut cmd = Cli::command();
    cmd = cmd.version(semver);

    let matches = cmd.get_matches();
    let cli = Cli::from_arg_matches(&matches)?;

    if cli.license {
        println!("Copyright (C) 2026 Alexander R. Croft");
        println!("License: GPL-3.0-or-later");
        return Ok(());
    }

    let Some(command) = cli.command else {
        // `arg_required_else_help = true` should normally prevent this,
        // but keep a deterministic fallback.
        Cli::command().print_help()?;
        println!();
        return Ok(());
    };

    match command {
        Commands::Pack { spec, out, payload, encoding, header, header_file } => {
            let mut spec_out: Option<PathBuf> = None;
            let mut spec_payload: Option<PathBuf> = None;
            let mut spec_encoding: Option<String> = None;
            let mut spec_header_file: Option<PathBuf> = None;
            let mut spec_header_fields: Option<Vec<HeaderField>> = None;

            if let Some(spec_path) = spec {
                let spec = self::spec::load_spec(&spec_path)?;
                spec_out = Some(spec.out);
                spec_payload = Some(spec.payload);
                spec_encoding = Some(spec.encoding);
                spec_header_file = spec.header_file;
                spec_header_fields = Some(self::spec::header_fields_from_map(&spec.header));
            }

            let out_path = out.or(spec_out).ok_or_else(|| anyhow::anyhow!(
                "missing --out (or spec.out)"
            ))?;
            let payload_path = payload.or(spec_payload).ok_or_else(|| anyhow::anyhow!(
                "missing --payload (or spec.payload)"
            ))?;

            let encoding_str = encoding
                .or(spec_encoding)
                .unwrap_or_else(|| "A".to_string());
            let encoding = self::spec::parse_encoding(&encoding_str)?;

            let payload_bytes = fs::read(&payload_path)
                .with_context(|| format!("read payload {}", payload_path.display()))?;

            let (header_fields, header_decoded) = match encoding {
                Encoding::Ascii => {
                    let mut fields: Vec<HeaderField> = if !header.is_empty() {
                        header
                            .into_iter()
                            .map(|(k, v)| HeaderField { key: k, value: v })
                            .collect()
                    } else {
                        spec_header_fields.unwrap_or_default()
                    };
                    // Canonicalize field order.
                    fields.sort_by(|a, b| a.key.cmp(&b.key));
                    (Some(fields), Vec::new())
                }
                Encoding::Base64 | Encoding::Cbor => {
                    let header_path = header_file
                        .or(spec_header_file)
                        .ok_or_else(|| anyhow::anyhow!("--header-file (or spec.header_file) is required for encoding B or C"))?;
                    let bytes = fs::read(&header_path)
                        .with_context(|| format!("read header file {}", header_path.display()))?;
                    (None, bytes)
                }
            };

            let capsule = Capsule::from_decoded(
                capsule_lib::Version(0x0001),
                encoding,
                header_fields.as_deref(),
                &header_decoded,
                &payload_bytes,
            )?;

            let bytes = capsule.to_bytes()?;
            fs::write(&out_path, bytes).with_context(|| format!("write {}", out_path.display()))?;
            Ok(())
        }

        Commands::Info { file, verify } => {
            let bytes = fs::read(&file).with_context(|| format!("read {}", file.display()))?;
            let decoded = Capsule::parse_with_options(
                &bytes,
                ParseOptions { verify_crc: verify, validate_encoding: verify },
            )?;

            println!("version={:04X}", decoded.prelude.version.0);
            println!("encoding={:?}", decoded.prelude.encoding);
            println!("header_len_encoded={}", decoded.header_encoded.len());
            println!("payload_len_encoded={}", decoded.payload_encoded.len());
            println!("header_len_decoded={}", decoded.header_decoded.len());
            println!("payload_len_decoded={}", decoded.payload_decoded.len());
            println!("body_crc_declared={:08X}", decoded.prelude.body_crc);
            if verify {
                let computed = capsule_lib::crc::compute_crc32_iso_hdlc(&bytes[24..]);
                println!("body_crc_computed={:08X}", computed);
            }

            if let Some(fields) = decoded.header_fields {
                for f in fields {
                    println!("header:{}={}", f.key, f.value);
                }
            }

            Ok(())
        }

        Commands::Verify { file } => {
            let bytes = fs::read(&file).with_context(|| format!("read {}", file.display()))?;
            Capsule::parse_with_options(
                &bytes,
                ParseOptions { verify_crc: true, validate_encoding: true },
            )?;
            println!("OK");
            Ok(())
        }

        Commands::Unpack { file, out_payload, out_header, verify, raw } => {
            let bytes = fs::read(&file).with_context(|| format!("read {}", file.display()))?;
            let decoded = Capsule::parse_with_options(
                &bytes,
                ParseOptions { verify_crc: verify, validate_encoding: verify },
            )?;

            let payload_bytes: &[u8] = if raw {
                &decoded.payload_encoded
            } else {
                &decoded.payload_decoded
            };

            fs::write(&out_payload, payload_bytes)
                .with_context(|| format!("write payload {}", out_payload.display()))?;

            if let Some(out_header) = out_header {
                let header_bytes: &[u8] = if raw {
                    &decoded.header_encoded
                } else {
                    &decoded.header_decoded
                };

                fs::write(&out_header, header_bytes)
                    .with_context(|| format!("write header {}", out_header.display()))?;
            }

            Ok(())
        }
    }
}
