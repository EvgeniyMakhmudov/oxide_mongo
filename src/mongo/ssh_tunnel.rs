use std::env;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use ssh2::{Channel, CheckResult, KnownHostFileKind, Session};

use crate::i18n::tr;
use crate::ui::connections::{SshAuthMethod, SshTunnelSettings, looks_like_private_key};

const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);
const LOOP_SLEEP: Duration = Duration::from_millis(10);
const READY_TIMEOUT: Duration = Duration::from_secs(10);
const CHANNEL_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const BUFFER_SIZE: usize = 16 * 1024;

#[derive(Debug)]
pub struct SshTunnel {
    local_port: u16,
    shutdown: Sender<()>,
    handle: Option<JoinHandle<()>>,
}

impl SshTunnel {
    pub fn start(
        settings: &SshTunnelSettings,
        remote_host: &str,
        remote_port: u16,
    ) -> Result<Self, String> {
        if !settings.enabled {
            return Err(String::from("SSH tunnel is not enabled"));
        }

        let settings = settings.clone();
        let remote_host = remote_host.to_string();
        let (ready_tx, ready_rx) = mpsc::channel();
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            let _ = run_tunnel(settings, remote_host, remote_port, shutdown_rx, ready_tx);
        });

        let ready = ready_rx
            .recv_timeout(READY_TIMEOUT)
            .map_err(|_| String::from("SSH tunnel initialization timed out"))?;

        match ready {
            Ok(port) => Ok(Self { local_port: port, shutdown: shutdown_tx, handle: Some(handle) }),
            Err(error) => {
                let _ = handle.join();
                Err(error)
            }
        }
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        let _ = self.shutdown.send(());
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_tunnel(
    settings: SshTunnelSettings,
    remote_host: String,
    remote_port: u16,
    shutdown_rx: Receiver<()>,
    ready_tx: Sender<Result<u16, String>>,
) -> Result<(), String> {
    let tcp = TcpStream::connect((settings.host.as_str(), settings.port))
        .map_err(|err| err.to_string())?;

    let mut session = Session::new().map_err(|err| err.to_string())?;
    session.set_tcp_stream(tcp);
    session.handshake().map_err(|err| err.to_string())?;
    verify_known_host(&session, &settings.host, settings.port)?;

    match settings.auth_method {
        SshAuthMethod::Password => {
            let password = settings.password.as_deref().unwrap_or_default();
            session
                .userauth_password(&settings.username, password)
                .map_err(|err| err.to_string())?;
        }
        SshAuthMethod::PrivateKey => {
            let key_input = settings.private_key.as_deref().unwrap_or_default().trim();
            let passphrase = settings.passphrase.as_deref().filter(|value| !value.is_empty());
            if looks_like_private_key(key_input) {
                userauth_private_key_memory(&session, &settings.username, key_input, passphrase)?;
            } else {
                session
                    .userauth_pubkey_file(
                        &settings.username,
                        None,
                        Path::new(key_input),
                        passphrase,
                    )
                    .map_err(|err| err.to_string())?;
            }
        }
    }

    if !session.authenticated() {
        return Err(String::from("SSH authentication failed"));
    }

    let _ = session.set_keepalive(true, KEEPALIVE_INTERVAL.as_secs() as u32);

    let listener = TcpListener::bind("127.0.0.1:0").map_err(|err| err.to_string())?;
    listener.set_nonblocking(true).map_err(|err| err.to_string())?;
    let local_port = listener.local_addr().map_err(|err| err.to_string())?.port();

    let _ = ready_tx.send(Ok(local_port));
    let _ = session.set_blocking(false);

    let mut last_keepalive = Instant::now();
    let mut pairs: Vec<ForwardPair> = Vec::new();
    let mut pending: Vec<PendingConnect> = Vec::new();

    loop {
        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        if last_keepalive.elapsed() >= KEEPALIVE_INTERVAL {
            let _ = session.keepalive_send();
            last_keepalive = Instant::now();
        }

        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    if stream.set_nonblocking(true).is_err() {
                        continue;
                    }

                    pending.push(PendingConnect::new(stream));
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }

        for index in (0..pending.len()).rev() {
            let mut channel = None;
            let mut remove = false;

            match session.channel_direct_tcpip(&remote_host, remote_port, None) {
                Ok(opened) => channel = Some(opened),
                Err(err) => {
                    if std::io::Error::from(err).kind() != std::io::ErrorKind::WouldBlock {
                        remove = true;
                    }
                }
            }

            if let Some(opened) = channel {
                let pending_conn = pending.swap_remove(index);
                pairs.push(ForwardPair::new(pending_conn.stream, opened));
                continue;
            }

            if remove || pending[index].since.elapsed() >= CHANNEL_CONNECT_TIMEOUT {
                pending.swap_remove(index);
            }
        }

        for index in (0..pairs.len()).rev() {
            if pairs[index].step() {
                pairs.swap_remove(index);
            }
        }

        thread::sleep(LOOP_SLEEP);
    }

    Ok(())
}

fn verify_known_host(session: &Session, host: &str, port: u16) -> Result<(), String> {
    let mut known_hosts = session.known_hosts().map_err(|err| err.to_string())?;
    let path = known_hosts_path()?;

    known_hosts
        .read_file(&path, KnownHostFileKind::OpenSSH)
        .map_err(|err| format!("{}: {}", tr("Failed to read SSH known_hosts"), err))?;

    let (key, _) =
        session.host_key().ok_or_else(|| String::from(tr("Failed to read SSH host key")))?;

    match known_hosts.check_port(host, port, key) {
        CheckResult::Match => Ok(()),
        CheckResult::Mismatch => Err(String::from(tr("SSH host key mismatch"))),
        CheckResult::NotFound => Err(String::from(tr("SSH host is not present in known_hosts"))),
        CheckResult::Failure => Err(String::from(tr("SSH known_hosts check failed"))),
    }
}

fn known_hosts_path() -> Result<PathBuf, String> {
    let home = env::var("HOME").map_err(|_| String::from(tr("SSH known_hosts file not found")))?;
    let path = PathBuf::from(home).join(".ssh").join("known_hosts");
    if !path.exists() {
        return Err(String::from(tr("SSH known_hosts file not found")));
    }
    Ok(path)
}

fn userauth_private_key_memory(
    session: &Session,
    username: &str,
    key_data: &str,
    passphrase: Option<&str>,
) -> Result<(), String> {
    #[cfg(unix)]
    {
        session
            .userauth_pubkey_memory(username, None, key_data, passphrase)
            .map_err(|err| err.to_string())
    }

    #[cfg(not(unix))]
    {
        let _ = session;
        let _ = username;
        let _ = key_data;
        let _ = passphrase;
        Err(String::from(tr("SSH private key text is not supported on this platform")))
    }
}

struct ForwardPair {
    local: TcpStream,
    channel: Channel,
    local_to_remote: Buffer,
    remote_to_local: Buffer,
    local_closed: bool,
    remote_closed: bool,
}

struct PendingConnect {
    stream: TcpStream,
    since: Instant,
}

impl PendingConnect {
    fn new(stream: TcpStream) -> Self {
        Self { stream, since: Instant::now() }
    }
}

impl ForwardPair {
    fn new(local: TcpStream, channel: Channel) -> Self {
        Self {
            local,
            channel,
            local_to_remote: Buffer::new(),
            remote_to_local: Buffer::new(),
            local_closed: false,
            remote_closed: false,
        }
    }

    fn step(&mut self) -> bool {
        self.pump_local_to_remote();
        self.pump_remote_to_local();

        if self.local_closed
            && self.remote_closed
            && self.local_to_remote.is_empty()
            && self.remote_to_local.is_empty()
        {
            let _ = self.channel.close();
            return true;
        }

        false
    }

    fn pump_local_to_remote(&mut self) {
        if !self.local_closed && self.local_to_remote.is_empty() {
            let mut buffer = [0u8; BUFFER_SIZE];
            match self.local.read(&mut buffer) {
                Ok(0) => {
                    self.local_closed = true;
                    let _ = self.channel.send_eof();
                }
                Ok(read) => {
                    self.local_to_remote.push(&buffer[..read]);
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => {
                    self.local_closed = true;
                    let _ = self.channel.send_eof();
                }
            }
        }

        if !self.local_to_remote.is_empty() {
            match self.channel.write(self.local_to_remote.pending()) {
                Ok(0) => {}
                Ok(written) => self.local_to_remote.consume(written),
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => {
                    self.remote_closed = true;
                }
            }
        }
    }

    fn pump_remote_to_local(&mut self) {
        if !self.remote_closed && self.remote_to_local.is_empty() {
            let mut buffer = [0u8; BUFFER_SIZE];
            match self.channel.read(&mut buffer) {
                Ok(0) => {
                    self.remote_closed = true;
                    let _ = self.local.shutdown(Shutdown::Write);
                }
                Ok(read) => {
                    self.remote_to_local.push(&buffer[..read]);
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => {
                    self.remote_closed = true;
                    let _ = self.local.shutdown(Shutdown::Write);
                }
            }
        }

        if !self.remote_to_local.is_empty() {
            match self.local.write(self.remote_to_local.pending()) {
                Ok(0) => {}
                Ok(written) => self.remote_to_local.consume(written),
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => {
                    self.local_closed = true;
                }
            }
        }
    }
}

struct Buffer {
    data: Vec<u8>,
    offset: usize,
}

impl Buffer {
    fn new() -> Self {
        Self { data: Vec::new(), offset: 0 }
    }

    fn is_empty(&self) -> bool {
        self.offset >= self.data.len()
    }

    fn push(&mut self, chunk: &[u8]) {
        if self.is_empty() {
            self.data.clear();
            self.offset = 0;
        }
        self.data.extend_from_slice(chunk);
    }

    fn pending(&self) -> &[u8] {
        &self.data[self.offset..]
    }

    fn consume(&mut self, amount: usize) {
        self.offset = self.offset.saturating_add(amount);
        if self.is_empty() {
            self.data.clear();
            self.offset = 0;
        }
    }
}
