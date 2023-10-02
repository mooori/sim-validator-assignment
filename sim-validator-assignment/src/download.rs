use clap::{Args, ValueEnum};
use std::fs::File;
use std::{io::Write, path::PathBuf};

use dl_validator_data::{NearProtocol, Protocol as DlValidatorDataProtocol};

use crate::validator::RawValidatorData;

#[derive(Args, Debug)]
pub(crate) struct DownloadConfig {
    /// The protocol for which to download data.
    #[arg(long, value_enum)]
    pub protocol: Protocol,
    /// URL of an RPC from which to download the data.
    #[arg(long)]
    pub rpc_url: String,
    /// Block height for which validator data is downloaded. If no value is provided the latest
    /// block will be queried.
    #[arg(long)]
    pub block_height: Option<u64>,
    /// The path of the file to which validator data will be written.
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Protocol {
    Near,
}

/// Downloads validator data, converts it to a vector of [`RawValidatorData`] and writes the
/// corresponding(pretty printed) JSON to the output file specified in `config`.
///
/// Pretty print JSON assuming users might want to inspect and modify validator data (mark
/// validators as malicious for simulations).
pub(crate) fn download(config: &DownloadConfig) -> anyhow::Result<()> {
    // Download validator data.
    let protocol = match config.protocol {
        Protocol::Near => NearProtocol::new(config.rpc_url.clone(), config.block_height),
    };
    let validator_data = protocol.download_validator_data()?;
    let validators: Vec<RawValidatorData> = validator_data.into_iter().map(|v| v.into()).collect();

    // Serialize it and write it to the output file.
    let pretty_json = serde_json::to_string_pretty(&validators)?;
    let mut file = File::create(config.out.as_path())?;
    file.write_all(pretty_json.as_bytes())?;

    Ok(())
}
