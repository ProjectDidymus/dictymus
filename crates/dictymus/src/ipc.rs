//! Single-instance IPC: the first instance owns a named pipe; later
//! instances forward their command line over it and exit.

use std::{
	env,
	path::{Path, PathBuf},
};

pub const IPC_COMMAND_ACTIVATE: &str = "ACTIVATE";
pub const SINGLE_INSTANCE_NAME: &str = "dictymus_running";

#[derive(Debug, Clone)]
pub enum IpcCommand {
	Activate,
	OpenFile(PathBuf),
}

#[cfg(any(windows, test))]
pub fn decode_execute_payload(data: &[u8]) -> Option<IpcCommand> {
	if data.is_empty() {
		return None;
	}
	let payload = String::from_utf8_lossy(data);
	let payload = payload.replace('\0', "");
	let payload = payload.trim();
	if payload.is_empty() {
		return None;
	}
	if payload == IPC_COMMAND_ACTIVATE {
		return Some(IpcCommand::Activate);
	}
	Some(IpcCommand::OpenFile(PathBuf::from(payload)))
}

/// Absolutize against the calling process's cwd; `dunce` keeps the result
/// free of `\\?\` verbatim prefixes.
pub fn normalize_cli_path(path: &Path) -> PathBuf {
	if let Ok(normalized) = dunce::canonicalize(path) {
		return normalized;
	}
	if path.is_absolute() {
		return path.to_path_buf();
	}
	env::current_dir().map_or_else(|_| path.to_path_buf(), |cwd| cwd.join(path))
}

/// Named pipe path scoped to the current user; the default pipe security
/// descriptor further restricts connections to the same user.
#[cfg(windows)]
pub fn named_pipe_path() -> String {
	let user = env::var("USERNAME").unwrap_or_else(|_| "user".to_string());
	format!(r"\\.\pipe\dictymus_{user}")
}

pub fn command_from_cli() -> IpcCommand {
	if let Some(path) = env::args().nth(1) {
		return IpcCommand::OpenFile(normalize_cli_path(Path::new(&path)));
	}
	IpcCommand::Activate
}

pub fn send_command(command: &IpcCommand) {
	tracing::debug!(?command, "sending IPC command to existing instance");
	let payload = match command {
		IpcCommand::Activate => IPC_COMMAND_ACTIVATE.to_string(),
		IpcCommand::OpenFile(path) => path.to_string_lossy().to_string(),
	};
	#[cfg(windows)]
	{
		use windows::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow;

		// ASFW_ANY: donate this process's foreground-activation right so the
		// running instance can bring itself forward.
		let _ = unsafe { AllowSetForegroundWindow(u32::MAX) };
		pipe::send(&named_pipe_path(), &payload);
	}
	#[cfg(not(windows))]
	let _ = payload;
}

/// Create the pipe server and dispatch decoded commands to the running app
/// on the wx main thread. Returns false when the pipe could not be created.
#[cfg(windows)]
pub fn start_server() -> bool {
	let name = named_pipe_path();
	let Some(handle) = pipe::try_create_server(&name) else {
		tracing::error!(pipe = %name, "failed to create IPC server; named pipe already exists");
		return false;
	};
	tracing::info!(pipe = %name, "IPC server started");
	pipe::serve_loop(handle, move |data| {
		if let Some(cmd) = decode_execute_payload(&data) {
			wxdragon::call_after(Box::new(move || {
				if let Some(app) = crate::app::app_from_ptr() {
					app.handle_ipc_command(cmd);
				}
			}));
			wxdragon::wake_up_idle();
		}
	});
	true
}

#[cfg(not(windows))]
pub fn start_server() -> bool {
	true
}

#[cfg(windows)]
mod pipe {
	use std::{ffi::OsStr, os::windows::ffi::OsStrExt as _};

	use windows::{
		Win32::{
			Foundation::{CloseHandle, ERROR_PIPE_CONNECTED, HANDLE},
			Storage::FileSystem::{
				CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_MODE,
				OPEN_EXISTING, ReadFile, WriteFile,
			},
			System::Pipes::{
				ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, NAMED_PIPE_MODE,
				WaitNamedPipeW,
			},
		},
		core::PCWSTR,
	};

	const BUF: usize = 4096;
	const GENERIC_WRITE: u32 = 0x4000_0000;
	const PIPE_ACCESS_INBOUND: u32 = 0x0000_0001;
	const PIPE_FLAG_FIRST_INSTANCE: u32 = 0x0008_0000; // FILE_FLAG_FIRST_PIPE_INSTANCE
	const PIPE_UNLIMITED_INSTANCES: u32 = 255;

	fn wide_nul(s: &str) -> Vec<u16> {
		OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
	}

	/// Try to create the server-side named pipe instance.
	/// Returns `None` when the pipe already exists (another instance is running).
	pub fn try_create_server(pipe_name: &str) -> Option<HANDLE> {
		let name = wide_nul(pipe_name);
		let handle = unsafe {
			CreateNamedPipeW(
				PCWSTR(name.as_ptr()),
				FILE_FLAGS_AND_ATTRIBUTES(PIPE_ACCESS_INBOUND | PIPE_FLAG_FIRST_INSTANCE),
				NAMED_PIPE_MODE(0), // PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT = 0
				PIPE_UNLIMITED_INSTANCES,
				0,
				BUF as u32,
				0,
				None,
			)
		};
		if handle.is_invalid() { None } else { Some(handle) }
	}

	/// Accept one connection, read the payload, disconnect, repeat.
	/// HANDLE is !Send; convert to raw usize so the closure can cross the thread boundary.
	pub fn serve_loop(handle: HANDLE, on_data: impl Fn(Vec<u8>) + Send + 'static) {
		let raw = handle.0 as usize;
		std::thread::spawn(move || {
			let h = HANDLE(raw as *mut _);
			loop {
				let conn = unsafe { ConnectNamedPipe(h, None) };
				let ready = conn.is_ok()
					|| unsafe { windows::Win32::Foundation::GetLastError() }
						== ERROR_PIPE_CONNECTED;
				if ready {
					let mut buf = vec![0u8; BUF];
					let mut n = 0u32;
					let ok = unsafe { ReadFile(h, Some(&mut buf), Some(&raw mut n), None) };
					if ok.is_ok() && n > 0 {
						on_data(buf[..n as usize].to_vec());
					}
				}
				let _ = unsafe { DisconnectNamedPipe(h) };
			}
		});
	}

	pub fn send(pipe_name: &str, payload: &str) {
		let name = wide_nul(pipe_name);
		// Allow up to 2 s for the server to become ready.
		let _ = unsafe { WaitNamedPipeW(PCWSTR(name.as_ptr()), 2000) };
		let Ok(file) = (unsafe {
			CreateFileW(
				PCWSTR(name.as_ptr()),
				GENERIC_WRITE,
				FILE_SHARE_MODE(0),
				None,
				OPEN_EXISTING,
				FILE_ATTRIBUTE_NORMAL,
				None, // hTemplateFile
			)
		}) else {
			return;
		};
		let _ = unsafe { WriteFile(file, Some(payload.as_bytes()), None, None) };
		let _ = unsafe { CloseHandle(file) };
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn decode_handles_empty_and_nulls() {
		assert!(decode_execute_payload(b"").is_none());
		assert!(decode_execute_payload(b"\0\0").is_none());
		assert!(decode_execute_payload(b" \0").is_none());
	}

	#[test]
	fn decode_handles_activate() {
		assert!(matches!(decode_execute_payload(b"ACTIVATE\0"), Some(IpcCommand::Activate)));
		assert!(matches!(decode_execute_payload(b"  ACTIVATE  "), Some(IpcCommand::Activate)));
	}

	#[test]
	fn decode_handles_open_file() {
		match decode_execute_payload(b"C:\\dicts\\bible.ifo\0") {
			Some(IpcCommand::OpenFile(path)) => {
				assert_eq!(path, PathBuf::from("C:\\dicts\\bible.ifo"))
			}
			other => panic!("expected OpenFile, got {other:?}"),
		}
	}

	#[test]
	fn decode_allows_spaced_paths() {
		match decode_execute_payload(b"  C:\\My Dicts\\lexicon.mdx  ") {
			Some(IpcCommand::OpenFile(path)) => {
				assert_eq!(path, PathBuf::from("C:\\My Dicts\\lexicon.mdx"))
			}
			other => panic!("expected OpenFile, got {other:?}"),
		}
	}

	#[test]
	fn decode_handles_non_utf8_bytes_lossy() {
		match decode_execute_payload(&[0xFF, 0xFE, b'a']) {
			Some(IpcCommand::OpenFile(path)) => assert!(path.to_string_lossy().contains('a')),
			other => panic!("expected OpenFile, got {other:?}"),
		}
	}

	#[test]
	fn normalize_handles_absolute_and_relative() {
		#[cfg(windows)]
		let abs = Path::new("C:\\nonexistent_abs_path");
		#[cfg(not(windows))]
		let abs = Path::new("/nonexistent_abs_path");
		assert_eq!(normalize_cli_path(abs), abs.to_path_buf());
		let rel = Path::new("nonexistent_rel_path");
		let expected = env::current_dir().unwrap().join(rel);
		assert_eq!(normalize_cli_path(rel), expected);
	}

	#[test]
	fn normalize_canonicalizes_existing_path() {
		let cwd = dunce::canonicalize(env::current_dir().unwrap()).unwrap();
		assert_eq!(normalize_cli_path(Path::new(".")), cwd);
	}
}
