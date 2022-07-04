use crate::geyser::load_plugin;
use clap::{ArgGroup, Parser};
use indicatif::{ProgressBar, ProgressBarIter, ProgressStyle};
use log::info;
use serde::Serialize;
use solana_geyser_plugin_interface::geyser_plugin_interface::{
    ReplicaAccountInfoV2, ReplicaAccountInfoVersions,
};
use solana_snapshot_etl::{ReadProgressTracking, UnpackedSnapshotLoader};
use std::io::{IoSliceMut, Read};
use std::path::{Path, PathBuf};

mod geyser;
mod mpl_metadata;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("action")
        .required(true)
        .args(&["geyser"]),
))]
struct Args {
    #[clap(help = "Path to snapshot")]
    path: String,
    #[clap(long, action, help = "Index token program data")]
    tokens: bool,
    #[clap(long, help = "Load Geyser plugin from given config file")]
    geyser: Option<String>,
}

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    if let Err(e) = _main() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn _main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    // $ Read snapshot file
    let loader =
        UnpackedSnapshotLoader::open_with_progress(&args.path, Box::new(LoadProgressTracking {}))?;
    // $ Dump to Geyser plugin
    if let Some(geyser_config_path) = args.geyser {
        info!("Dumping to Geyser plugin: {}", &geyser_config_path);
        let mut plugin = unsafe { load_plugin(&geyser_config_path)? };
        assert!(
            plugin.account_data_notifications_enabled(),
            "Geyser plugin does not accept account data notifications"
        );
        // TODO dedup spinner definitions
        let spinner_style = ProgressStyle::with_template(
            "{prefix:>10.bold.dim} {spinner} rate={per_sec}/s total={human_pos}",
        )
        .unwrap();
        let accounts_spinner = ProgressBar::new_spinner()
            .with_style(spinner_style.clone())
            .with_prefix("accs");
        let mut accounts_count = 0u64;
        // # Deserialize account info from snapshot file (loader)
        for account in loader.iter() {
            let account = account?;
            let account = account.access().unwrap();
            let slot = 0u64; // TODO fix slot number
            plugin.update_account(
                ReplicaAccountInfoVersions::V0_0_2(&ReplicaAccountInfoV2 {
                    pubkey: account.meta.pubkey.as_ref(),
                    lamports: account.account_meta.lamports,
                    owner: account.account_meta.owner.as_ref(),
                    executable: account.account_meta.executable,
                    rent_epoch: account.account_meta.rent_epoch,
                    data: account.data,
                    write_version: account.meta.write_version,
                    txn_signature: None,
                }),
                slot,
                /* is_startup */ false,
            )?;
            accounts_count += 1;
            if accounts_count % 1024 == 0 {
                accounts_spinner.set_position(accounts_count);
            }
        }
        accounts_spinner.finish();
        println!("Done!");
    }
    Ok(())
}

struct LoadProgressTracking {}

impl ReadProgressTracking for LoadProgressTracking {
    fn new_read_progress_tracker(
        &self,
        _: &Path,
        rd: Box<dyn Read>,
        file_len: u64,
    ) -> Box<dyn Read> {
        let progress_bar = ProgressBar::new(file_len).with_style(
            ProgressStyle::with_template(
                "{prefix:>10.bold.dim} {spinner:.green} [{bar:.cyan/blue}] {bytes}/{total_bytes} ({percent}%)",
            )
            .unwrap()
            .progress_chars("#>-"),
        );
        progress_bar.set_prefix("manifest");
        Box::new(LoadProgressTracker {
            rd: progress_bar.wrap_read(rd),
            progress_bar,
        })
    }
}

struct LoadProgressTracker {
    progress_bar: ProgressBar,
    rd: ProgressBarIter<Box<dyn Read>>,
}

impl Drop for LoadProgressTracker {
    fn drop(&mut self) {
        self.progress_bar.finish()
    }
}

impl Read for LoadProgressTracker {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.rd.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.rd.read_vectored(bufs)
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.rd.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.rd.read_exact(buf)
    }
}

#[derive(Serialize)]
struct CSVRecord {
    pubkey: String,
    owner: String,
    data_len: u64,
    lamports: u64,
}
