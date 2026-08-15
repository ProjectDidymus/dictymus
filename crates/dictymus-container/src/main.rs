//! Publisher CLI for `.dicty` containers and `.dictykey` licenses.

use clap::{Parser, Subcommand};
use dictymus_container::container::{collect_stardict_files, ifo_bookname, pack, seal};
use dictymus_container::keys::{load_or_create_scope_key, load_signing_key, write_signing_key};
use dictymus_container::license::issue;
use ed25519_dalek::SigningKey;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(about = "Package and license Dictymus dictionaries")]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	/// Generate a publisher signing keypair.
	Keygen {
		/// Path of the private keyfile to create; the public key is
		/// written next to it with a .pub extension and printed.
		out: PathBuf,
	},
	/// Build an unsealed container from a StarDict .ifo.
	Pack {
		ifo: PathBuf,
		#[arg(long)]
		id: Option<String>,
		#[arg(long)]
		name: Option<String>,
		#[arg(short, long)]
		out: Option<PathBuf>,
	},
	/// Build a sealed container from a StarDict .ifo.
	Seal {
		ifo: PathBuf,
		/// Scope keyfile (repeatable; created if absent). The file
		/// stem is the scope id: suite.dek grants scope "suite".
		#[arg(long = "scope-key", required = true)]
		scope_keys: Vec<PathBuf>,
		#[arg(long)]
		id: Option<String>,
		#[arg(long)]
		name: Option<String>,
		#[arg(short, long)]
		out: Option<PathBuf>,
	},
	/// Issue a signed license for one or more scopes.
	License {
		#[arg(long = "signing-key")]
		signing_key: PathBuf,
		#[arg(long = "scope-key", required = true)]
		scope_keys: Vec<PathBuf>,
		#[arg(long)]
		licensee: String,
		/// Issue date; defaults to today.
		#[arg(long)]
		issued: Option<String>,
		#[arg(short, long)]
		out: PathBuf,
	},
	/// Show the public fields of a container or license file.
	Inspect { file: PathBuf },
}

type Fileset = Vec<(String, Vec<u8>)>;

fn scope_id_of(path: &Path) -> Result<String, String> {
	path.file_stem()
		.and_then(|s| s.to_str())
		.map(str::to_string)
		.ok_or_else(|| format!("scope keyfile {} has no usable file stem", path.display()))
}

fn load_scopes(paths: &[PathBuf]) -> Result<Vec<(String, [u8; 32])>, String> {
	paths
		.iter()
		.map(|p| {
			let key = load_or_create_scope_key(p)
				.map_err(|e| format!("reading scope key {}: {e}", p.display()))?;
			Ok((scope_id_of(p)?, key))
		})
		.collect()
}

/// Container id, display name, output path and fileset for pack/seal,
/// defaulting the id to the lowercased stem, the name to the .ifo
/// bookname, and the output to `<stem>.dicty` next to the input.
fn container_args(
	ifo: &Path,
	id: Option<String>,
	name: Option<String>,
	out: Option<PathBuf>,
) -> Result<(String, String, PathBuf, Fileset), String> {
	let files = collect_stardict_files(ifo).map_err(|e| e.to_string())?;
	let stem = ifo
		.file_stem()
		.and_then(|s| s.to_str())
		.ok_or_else(|| format!("{} has no usable file stem", ifo.display()))?;
	let id = id.unwrap_or_else(|| stem.to_lowercase());
	let name =
		name.or_else(|| ifo_bookname(&files[0].1)).ok_or("no bookname in the .ifo; pass --name")?;
	let out = out.unwrap_or_else(|| ifo.with_extension("dicty"));
	Ok((id, name, out, files))
}

fn run() -> Result<(), String> {
	match Cli::parse().command {
		Command::Keygen { out } => {
			let key = SigningKey::generate(&mut rand::rngs::OsRng);
			write_signing_key(&out, &key).map_err(|e| e.to_string())?;
			let public: String =
				key.verifying_key().to_bytes().iter().map(|b| format!("{b:02x}")).collect();
			std::fs::write(out.with_extension("pub"), format!("{public}\n"))
				.map_err(|e| e.to_string())?;
			println!("public key (embed in the app): {public}");
		}
		Command::Pack { ifo, id, name, out } => {
			let (id, name, out, files) = container_args(&ifo, id, name, out)?;
			std::fs::write(&out, pack(&id, &name, &files)).map_err(|e| e.to_string())?;
			println!("packed {name} (id {id}, unsealed) into {}", out.display());
		}
		Command::Seal { ifo, scope_keys, id, name, out } => {
			let scopes = load_scopes(&scope_keys)?;
			let (id, name, out, files) = container_args(&ifo, id, name, out)?;
			std::fs::write(&out, seal(&id, &name, &files, &scopes)).map_err(|e| e.to_string())?;
			let scope_ids: Vec<&str> = scopes.iter().map(|(s, _)| s.as_str()).collect();
			println!(
				"sealed {name} (id {id}, scopes: {}) into {}",
				scope_ids.join(", "),
				out.display()
			);
		}
		Command::License { signing_key, scope_keys, licensee, issued, out } => {
			let signing = load_signing_key(&signing_key)
				.map_err(|e| format!("reading signing key {}: {e}", signing_key.display()))?;
			let grants = load_scopes(&scope_keys)?;
			let issued =
				issued.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
			std::fs::write(&out, issue(&licensee, &issued, &grants, &signing))
				.map_err(|e| e.to_string())?;
			println!("issued license for {licensee} to {}", out.display());
		}
		Command::Inspect { file } => {
			let bytes = std::fs::read(&file).map_err(|e| e.to_string())?;
			print!("{}", dictymus_container::inspect(&bytes).map_err(|e| e.to_string())?);
		}
	}
	Ok(())
}

fn main() -> ExitCode {
	match run() {
		Ok(()) => ExitCode::SUCCESS,
		Err(message) => {
			eprintln!("error: {message}");
			ExitCode::FAILURE
		}
	}
}
