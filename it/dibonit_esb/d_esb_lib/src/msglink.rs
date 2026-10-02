// ============================================================================
// msglink — message exchange library between applications on the same host
// ============================================================================
//
// Design choices:
//
//  Transport: Unix Domain Socket (UDS, tokio). Both apps run on the same
//  server, so UDS is faster, safer (no network stack, no open port) and
//  simpler than TCP localhost. The filesystem already restricts who may
//  connect via socket file permissions (0o600).
//
//  Authentication: name + password verified by a server-side callback
//  (store hashes, see the argon2 example in the `Verifier` docs below).
//  SSH keys are not used here: they suit humans, not machine-to-machine IPC.
//  For a networked deployment, swap in a TLS/mTLS handshake or an HMAC
//  challenge-response (extension point noted in the code).
//
//  Authorization: after authentication, srv issues a 256-bit random session
//  token tied to the connection, with a TTL. Every subsequent message must
//  carry it; srv rejects unknown/expired tokens. Tokens are bound to their
//  socket, so app1 cannot impersonate app2 (or srv) and vice versa.
//
//  Concurrency: each accepted connection gets its own reader task plus a
//  writer task fed by an mpsc channel. srv never blocks while serving one
//  client, so several apps can exchange messages in parallel without losing
//  messages. srv can also initiate requests towards a connected app
//  (Ctx::get_from / multi-hop test below).
//
//  Protocol (JSON frames, 4-byte big-endian length prefix):
//
//    app  -> srv : Hello { name, password }
//    srv  -> app : Welcome { contracts, token }   | Error { reason }
//    app  -> srv : Get { contract, id_request, token }
//    srv  -> app : Ack { id_request } then Give { id_request, data }
//    x -> y      : Publish { contract, id_request, token }
//    y -> x      : Ack { id_request } then Published { id_request }
//    srv  -> app : Get { contract, id_request, token }  (server-initiated;
//                  the app answers with Ack + Give if it runs a ClientHandler)
//
//  Cargo.toml:
//
//    [dependencies]
//    tokio = { version = "1", features = ["full"] }
//    serde = { version = "1", features = ["derive"] }
//    serde_json = "1"
//    rand = "0.8"
//    async-trait = "0.1"
//
//  Minimal usage (two binaries sharing this lib):
//
//    // srv.rs
//    #[tokio::main]
//    async fn main() -> Result<(), Box<dyn std::error::Error>> {
//        let server = msglink::Server::bind("/tmp/msglink.sock").await?;
//        server.run(MyHandler::new()).await?;     // accept loop
//        Ok(())
//    }
//
//    // app1.rs
//    #[tokio::main]
//    async fn main() -> Result<(), Box<dyn std::error::Error>> {
//        let (conn, contracts) = msglink::Client::connect("/tmp/msglink.sock",
//            "app1", "s3cret", None).await?;  // Err if srv is not running
//        conn.get("weather", "req-1").await?; // -> Ack then Give
//        Ok(())
//    }
// ============================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, Mutex};

/// Session token lifetime in seconds. Expired => re-authenticate.
pub const SESSION_TTL_SECS: u64 = 3600;

/// Timeout for waiting on a remote answer (Ack/Give/Published).
const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum Error {
    /// srv is not running (socket missing) or connection refused.
    NotRunning(std::io::Error),
    /// Authentication rejected by srv (bad name/password).
    Auth(String),
    /// Message received with a missing, invalid or expired token.
    Unauthorized,
    /// Framing/JSON/IO failure during an exchange.
    Io(std::io::Error),
    /// Semantically invalid request (unknown contract, timeout, ...).
    Protocol(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotRunning(_) => write!(f, "server is not running"),
            Error::Auth(r) => write!(f, "authentication refused: {r}"),
            Error::Unauthorized => write!(f, "invalid or expired session token"),
            Error::Io(e) => write!(f, "exchange error: {e}"),
            Error::Protocol(r) => write!(f, "protocol error: {r}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::ConnectionRefused
            | std::io::ErrorKind::NotFound
            | std::io::ErrorKind::PermissionDenied => Error::NotRunning(e),
            _ => Error::Io(e),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// ---------------------------------------------------------------------------
// Protocol
// ---------------------------------------------------------------------------

/// Opaque token type (hex, 256 random bits); not guessable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Token(String);

impl Token {
    fn new_random() -> Self {
        use rand::RngCore;
        let mut b = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut b);
        Token(b.iter().map(|x| format!("{x:02x}")).collect())
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug,Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    /// app -> srv: connection request.
    Hello { name: String, password: String },
    /// srv -> app: success + available contracts + session token.
    Welcome { contracts: Vec<String>, token: Token },
    /// Generic error message (auth, unknown contract, handler failure, ...).
    Error { reason: String },
    /// Data request on a contract. Token is mandatory.
    Get { contract: String, id_request: String, token: Token },
    /// Reception acknowledgement for a request identified by id_request.
    Ack { id_request: String },
    /// Data answer to a Get.
    Give { id_request: String, data: Vec<u8> },
    /// Publication on a contract. Token is mandatory.
    Publish { contract: String, id_request: String, token: Token },
    /// Publication confirmation.
    Published { id_request: String },
}

// --- frames: 4 BE length bytes + JSON --------------------------------------

async fn send_msg<S: AsyncWriteExt + Unpin>(s: &mut S, m: &Message) -> Result<()> {
    let json =
        serde_json::to_vec(m).map_err(|e| Error::Protocol(format!("serialize: {e}")))?;
    s.write_all(&(json.len() as u32).to_be_bytes()).await?;
    s.write_all(&json).await?;
    s.flush().await?;
    Ok(())
}

async fn recv_msg<S: AsyncReadExt + Unpin>(s: &mut S) -> Result<Message> {
    let mut len = [0u8; 4];
    s.read_exact(&mut len).await?;
    let mut buf = vec![0u8; u32::from_be_bytes(len) as usize];
    s.read_exact(&mut buf).await?;
    serde_json::from_slice(&buf)
        .map_err(|e| Error::Protocol(format!("deserialize: {e}")))
}

/// Waits for the next message on a pending channel, with timeout.
async fn next_reply(rx: &mut mpsc::UnboundedReceiver<Message>, what: &str) -> Result<Message> {
    tokio::time::timeout(REPLY_TIMEOUT, rx.recv())
        .await
        .map_err(|_| Error::Protocol(format!("timeout waiting for {what}")))?
        .ok_or_else(|| Error::Protocol(format!("peer closed before {what}")))
}

// ---------------------------------------------------------------------------
// Shared broker: sessions, per-app writer channels, pending server requests
// ---------------------------------------------------------------------------

struct Session {
    name: String,
    expires_at: std::time::Instant,
}

/// Handle given to server handlers so their business logic can talk back to
/// any connected app (e.g. srv asks app2 for data while serving app1).
pub struct Ctx {
    broker: Arc<Broker>,
}

impl Ctx {
    /// Names of the currently connected apps.
    pub async fn apps(&self) -> Vec<String> {
        self.broker.writers.lock().await.keys().cloned().collect()
    }

    /// Server-initiated request: send Get to `app`, wait for its Ack then Give,
    /// and return the data. Fails if `app` is not connected or does not answer
    /// in time. This runs concurrently with other connections, so srv keeps
    /// serving other apps while waiting for `app`.
    pub async fn get_from(&self, app: &str, contract: &str, id_request: &str)
        -> Result<Vec<u8>> {
        self.broker.get_from(app, contract, id_request).await
    }
}

struct Broker {
    /// token -> session (name, expiry).
    sessions: Mutex<HashMap<String, Session>>,
    /// app name -> writer channel (one per authenticated connection).
    writers: Mutex<HashMap<String, mpsc::Sender<Message>>>,
    /// id_request -> channel used to deliver the answers (Ack, Give, ...)
    /// of a server-initiated request back to its issuer.
    pending: Mutex<HashMap<String, mpsc::UnboundedSender<Message>>>,
}

impl Broker {
    /// Session token bound to a given app name, if any.
    async fn token_of(&self, name: &str) -> Option<Token> {
        let sessions = self.sessions.lock().await;
        sessions
            .iter()
            .find(|(_, s)| s.name == name && s.expires_at > std::time::Instant::now())
            .map(|(t, _)| Token(t.clone()))
    }

    async fn get_from(&self, app: &str, contract: &str, id_request: &str)
        -> Result<Vec<u8>> {
        // Resolve the target app: it must be connected with a valid session.
        let (token, writer) = match self.token_of(app).await {
            Some(t) => {
                let writers = self.writers.lock().await;
                match writers.get(app) {
                    Some(w) => (t, w.clone()),
                    None => return Err(Error::Protocol(format!("app {app:?} not connected"))),
                }
            }
            None => {
                return Err(Error::Protocol(format!("app {app:?} not connected")))
            }
        };

        // Register the answer channel before sending, so no reply is lost.
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.pending
            .lock()
            .await
            .insert(id_request.to_string(), tx);
        writer
            .send(Message::Get {
                contract: contract.to_string(),
                id_request: id_request.to_string(),
                token,
            })
            .await
            .map_err(|_| Error::Protocol(format!("app {app:?} disconnected")))?;

        // Expect Ack(id_request) then Give(id_request, data).
        let res = async {
            match next_reply(&mut rx, "Ack").await? {
                Message::Ack { id_request: id } if id == id_request => {}
                Message::Error { reason } => return Err(Error::Protocol(reason)),
                _ => return Err(Error::Protocol("Ack expected".into())),
            }
            match next_reply(&mut rx, "Give").await? {
                Message::Give { id_request: id, data } if id == id_request => Ok(data),
                Message::Error { reason } => Err(Error::Protocol(reason)),
                _ => Err(Error::Protocol("Give expected".into())),
            }
        }
        .await;

        // Defensive cleanup in case of timeout/error so the map stays small.
        self.pending.lock().await.remove(id_request);
        res
    }

    /// Route an answer (Ack/Give/Published/Error) coming from a client
    /// to the pending server-initiated request with the same id_request,
    /// if any. Error frames carry no id, so they are broadcast to every
    /// pending request. Unknown answers are silently dropped.
    async fn route_answer(&self, msg: &Message) {
        // Give/Published terminate the exchange; Ack just passes through.
        let terminal = !matches!(msg, Message::Ack { .. });
        if let Message::Error { reason } = msg {
            // Broadcast the error to all pending requests, then clear.
            let map = self.pending.lock().await;
            for (_, tx) in map.iter() {
                let _ = tx.send(Message::Error { reason: reason.clone() });
            }
            drop(map);
            self.pending.lock().await.clear();
            return;
        }
        let id = match &msg {
            Message::Ack { id_request }
            | Message::Give { id_request, .. }
            | Message::Published { id_request } => id_request.clone(),
            _ => return,
        };
        let map = self.pending.lock().await;
        if let Some(tx) = map.get(&id) {
            let _ = tx.send(msg.clone());
            if terminal {
                drop(map);
                self.pending.lock().await.remove(&id);
            }
        }
    }

    /// Cleanup on connection close.
    async fn disconnect(&self, token: &Token, name: &str) {
        self.sessions.lock().await.remove(token.as_str());
        self.writers.lock().await.remove(name);
    }
}

// ---------------------------------------------------------------------------
// Server side (srv)
// ---------------------------------------------------------------------------

/// Business callbacks provided by the srv application.
///
/// Password security: NEVER store passwords in clear. Inside `verify`,
/// compare against a hash, e.g. with the `argon2` crate:
///
///     use argon2::{Argon2, PasswordHash, PasswordVerifier};
///     let parsed = PasswordHash::new(&self.stored_hash)?;
///     Argon2::default()
///         .verify_password(password.as_bytes(), &parsed)
///         .is_ok()
///
/// (stored_hash = result of `Argon2::hash_password` at account creation).
#[async_trait]
pub trait Handler: Send + Sync + 'static {
    /// Checks name + password. false => Error::Auth on the app side.
    fn verify(&self, name: &str, password: &str) -> bool;

    /// Contracts offered; sent after authentication.
    fn contracts(&self) -> Vec<String>;

    /// Serves a Get. `ctx` lets the handler request data from another
    /// connected app (see Ctx::get_from). Ok(data) is sent as `Give`.
    async fn get(&self, ctx: &Ctx, caller: &str, contract: &str, id_request: &str)
        -> std::result::Result<Vec<u8>, String>;

    /// Serves a Publish. Ok(()) => `Published` is sent.
    async fn publish(&self, ctx: &Ctx, caller: &str, contract: &str, id_request: &str)
        -> std::result::Result<(), String>;
}

/// The server. One instance handles any number of concurrent connections:
/// each accepted socket gets its own reader task and writer task, and all
/// state (sessions, writers, pending requests) lives in a shared Broker.
pub struct Server {
    path: PathBuf,
    listener: UnixListener,
    broker: Arc<Broker>,
}

impl Server {
    /// Creates the socket. Removes a stale socket file left by a previous
    /// srv that died abruptly before binding.
    pub async fn bind<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if path.exists() {
            tokio::fs::remove_file(&path).await?;
        }
        let listener = UnixListener::bind(&path)?;
        // Restrict the socket to the current user (0o600) on unix:
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        tokio::fs::set_permissions(&path, perms).await?;
        Ok(Self {
            path,
            listener,
            broker: Arc::new(Broker {
                sessions: Mutex::new(HashMap::new()),
                writers: Mutex::new(HashMap::new()),
                pending: Mutex::new(HashMap::new()),
            }),
        })
    }

    /// Accept loop: spawn this in a task, or call it from main directly.
    pub async fn run<H: Handler>(self, handler: Arc<H>) -> Result<()> {
        loop {
            let (stream, _addr) = match self.listener.accept().await {
                Ok(v) => v,
                Err(e) => return Err(e.into()),
            };
            let broker = self.broker.clone();
            let handler = handler.clone();
            // One task per connection: requests are processed in parallel
            // without blocking each other or losing messages.
            tokio::spawn(async move {
                if let Err(e) = handle_conn(stream, handler, broker).await {
                    eprintln!("[srv] connection closed: {e}");
                }
            });
        }
    }

    /// Socket path (useful for logging).
    pub fn path(&self) -> &Path {
        &self.path
    }
}

async fn handle_conn<H: Handler>(
    stream: UnixStream,
    handler: Arc<H>,
    broker: Arc<Broker>,
) -> Result<()> {
    // Split the socket: the read half stays in this task, the write half is
    // owned by a dedicated writer task. This way responses, error frames and
    // server-initiated requests can be queued from several tasks safely.
    let (mut read_half, write_half) = stream.into_split();
    let (wtx, mut wrx) = mpsc::channel::<Message>(64);
    tokio::spawn(async move {
        let mut write_half = write_half;
        while let Some(msg) = wrx.recv().await {
            if send_msg(&mut write_half, &msg).await.is_err() {
                break;
            }
        }
    });

    // 1) Authentication. No Hello => disconnect.
    let (name, password) = match recv_msg(&mut read_half).await? {
        Message::Hello { name, password } => (name, password),
        _ => {
            let _ = wtx
                .send(Message::Error { reason: "Hello expected".into() })
                .await;
            return Err(Error::Protocol("first message was not Hello".into()));
        }
    };
    if !handler.verify(&name, &password) {
        // Security: generic message on the wire, details only in srv logs.
        let _ = wtx
            .send(Message::Error { reason: "authentication refused".into() })
            .await;
        return Err(Error::Auth(format!("credentials rejected for {name:?}")));
    }

    // 2) Issue the session token + contract list, register the writer.
    let token = Token::new_random();
    let contracts = handler.contracts();
    broker.sessions.lock().await.insert(
        token.as_str().to_string(),
        Session {
            name: name.clone(),
            expires_at: std::time::Instant::now()
                + std::time::Duration::from_secs(SESSION_TTL_SECS),
        },
    );
    broker.writers.lock().await.insert(name.clone(), wtx.clone());
    wtx.send(Message::Welcome {
        contracts,
        token: token.clone(),
    })
    .await
    .map_err(|_| Error::Protocol("client vanished during handshake".into()))?;

    // 3) Service loop. The token is checked BEFORE any processing.
    loop {
        let req = match recv_msg(&mut read_half).await {
            Ok(m) => m,
            Err(Error::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // clean disconnect
                broker.disconnect(&token, &name).await;
                return Ok(());
            }
            Err(e) => {
                broker.disconnect(&token, &name).await;
                return Err(e);
            }
        };

        match &req {
            // Answers to server-initiated requests: route to the issuer.
            Message::Ack { .. } | Message::Give { .. } | Message::Published { .. } => {
                broker.route_answer(&req).await;
                continue;
            }
            // Requests: the token must match this connection's session.
            Message::Get { token: t, .. } | Message::Publish { token: t, .. } => {
                let now = std::time::Instant::now();
                let ok = {
                    let map = broker.sessions.lock().await;
                    map.get(t.as_str())
                        .map(|s| t == &token && s.expires_at > now)
                        .unwrap_or(false)
                };
                if !ok {
                    let _ = wtx
                        .send(Message::Error { reason: "invalid or expired token".into() })
                        .await;
                    broker.disconnect(&token, &name).await;
                    return Err(Error::Unauthorized);
                }
            }
            _ => {
                let _ = wtx
                    .send(Message::Error { reason: "unexpected message".into() })
                    .await;
                broker.disconnect(&token, &name).await;
                return Err(Error::Unauthorized);
            }
        }

        let (contract, id_request) = match &req {
            Message::Get { contract, id_request, .. }
            | Message::Publish { contract, id_request, .. } => (contract, id_request),
            _ => unreachable!(),
        };

        match req {
            Message::Get { .. } => {
                // Ack(id_request) then Give(id_request, data)
                let _ = wtx
                    .send(Message::Ack { id_request: id_request.clone() })
                    .await;
                let ctx = Ctx { broker: broker.clone() };
                match handler.get(&ctx, &name, &contract, &id_request).await {
                    Ok(data) => {
                        let _ = wtx.send(Message::Give { id_request: id_request.clone() , data }).await;
                    }
                    Err(reason) => {
                        let _ = wtx.send(Message::Error { reason }).await;
                    }
                }
            }
            Message::Publish { .. } => {
                // Ack(id_request) then Published(id_request)
                let _ = wtx
                    .send(Message::Ack { id_request: id_request.clone() })
                    .await;
                let ctx = Ctx { broker: broker.clone() };
                match handler.publish(&ctx, &name, &contract, &id_request).await {
                    Ok(()) => {
                        let _ = wtx.send(Message::Published {  id_request: id_request.clone() }).await;
                    }
                    Err(reason) => {
                        let _ = wtx.send(Message::Error { reason }).await;
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

// ---------------------------------------------------------------------------
// Client side (app1, app2, ...)
// ---------------------------------------------------------------------------

/// Business callbacks for apps that also SERVE requests initiated by srv
/// (e.g. app2 answering a server-side Get). Pure clients can pass None.
#[async_trait]
pub trait ClientHandler: Send + Sync + 'static {
    /// Serves a Get coming from srv. Ok(data) is sent back as `Give`.
    async fn get(&self, contract: &str, id_request: &str)
        -> std::result::Result<Vec<u8>, String>;

    /// Serves a Publish coming from srv. Ok(()) => `Published` is sent.
    async fn publish(&self, contract: &str, id_request: &str)
        -> std::result::Result<(), String>;
}

/// Authenticated connection towards srv. Obtained through `Client::connect`.
#[derive(Debug)]
pub struct Client {
    tx: mpsc::Sender<Message>,
    /// id_request -> channel receiving the frames that answer our requests.
    pending: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Message>>>>,
    pub token: Token,
    /// Contract list received in the Welcome.
    pub contracts: Vec<String>,
}

impl Client {
    /// Connects and authenticates (Hello), then checks the reply.
    /// - Err(NotRunning): srv is not running / socket unreachable.
    /// - Err(Auth): credentials rejected.
    /// `server_handler` lets the app serve srv-initiated requests too
    /// (multi-hop scenario); pass None for a pure client.
    /// Ok: (client, contracts) — the token is already managed by the lib.
    pub async fn connect<P: AsRef<Path>>(
        path: P,
        name: &str,
        password: &str,
        server_handler: Option<Arc<dyn ClientHandler>>,
    ) -> Result<(Self, Vec<String>)> {
        let stream = UnixStream::connect(path).await?; // NotRunning on failure
        let (mut read_half, write_half) = stream.into_split();

        // Dedicated writer task: sends everything queued on `tx`.
        let (tx, mut wrx) = mpsc::channel::<Message>(64);
        let wtx = tx.clone();
        tokio::spawn(async move {
            let mut write_half = write_half;
            while let Some(msg) = wrx.recv().await {
                if send_msg(&mut write_half, &msg).await.is_err() {
                    break;
                }
            }
        });

        wtx.send(Message::Hello {
            name: name.into(),
            password: password.into(),
        })
        .await
        .map_err(|_| Error::Protocol("server vanished during handshake".into()))?;

        let (token, contracts) = match recv_msg(&mut read_half).await? {
            Message::Welcome { contracts, token } => (token, contracts),
            Message::Error { reason } => return Err(Error::Auth(reason)),
            _ => return Err(Error::Protocol("Welcome or Error expected".into())),
        };

        let client = Self {
            tx,
            pending: Arc::new(Mutex::new(HashMap::new())),
            token: token.clone(),
            contracts: contracts.clone(),
        };

        // Reader task: processes BOTH the answers to our own requests and
        // the requests initiated by srv (Get/Publish carrying our token).
        let pending = client.pending.clone();
        let my_token = token.clone();
        let responder = wtx.clone();
        tokio::spawn(async move {
            loop {
                match recv_msg(&mut read_half).await {
                    Ok(Message::Get { contract, id_request, token })
                        if token == my_token =>
                    {
                        if let Some(h) = &server_handler {
                            let _ = responder.send(Message::Ack {
                                id_request: id_request.clone(),
                            }).await;
                            match h.get(&contract, &id_request).await {
                                Ok(data) => {
                                    let _ = responder
                                        .send(Message::Give { id_request, data })
                                        .await;
                                }
                                Err(reason) => {
                                    let _ = responder
                                        .send(Message::Error { reason })
                                        .await;
                                }
                            }
                        } else {
                            let _ = responder
                                .send(Message::Error {
                                    reason: "unsupported request".into(),
                                })
                                .await;
                        }
                    }
                    Ok(Message::Publish { contract, id_request, token })
                        if token == my_token =>
                    {
                        if let Some(h) = &server_handler {
                            let _ = responder.send(Message::Ack {
                                id_request: id_request.clone(),
                            }).await;
                            match h.publish(&contract, &id_request).await {
                                Ok(()) => {
                                    let _ = responder
                                        .send(Message::Published { id_request })
                                        .await;
                                }
                                Err(reason) => {
                                    let _ = responder
                                        .send(Message::Error { reason })
                                        .await;
                                }
                            }
                        } else {
                            let _ = responder
                                .send(Message::Error {
                                    reason: "unsupported request".into(),
                                })
                                .await;
                        }
                    }
                    // Answers to our own requests: forward to the pending
                    // channel of the matching id_request.
                    Ok(msg @ (Message::Ack { .. }
                    | Message::Give { .. }
                    | Message::Published { .. }
                    | Message::Error { .. })) =>
                    {
                        // Error frames carry no id: broadcast to every
                        // pending request and clear the map.
                        if let Message::Error { reason } = &msg {
                            let map = pending.lock().await;
                            for (_, ch) in map.iter() {
                                let _ = ch.send(Message::Error {
                                    reason: reason.clone(),
                                });
                            }
                            drop(map);
                            pending.lock().await.clear();
                            continue;
                        }
                        let id = match &msg {
                            Message::Ack { id_request }
                            | Message::Give { id_request, .. }
                            | Message::Published { id_request } => {
                                Some(id_request.clone())
                            }
                            _ => None,
                        };
                        if let Some(id) = id {
                            // Give/Published terminate the exchange;
                            // compute before moving msg into send().
                            let terminal = !matches!(msg, Message::Ack { .. });
                            let map = pending.lock().await;
                            if let Some(ch) = map.get(&id) {
                                let _ = ch.send(msg);
                                drop(map);
                                if terminal {
                                    pending.lock().await.remove(&id);
                                }
                            }
                        }
                    }
                    Ok(_) => {} // unexpected frame: ignore
                    Err(_) => return, // socket closed: stop the reader
                }
            }
        });

        Ok((client, contracts))
    }

    /// Returns a new handle sharing the same underlying connection and
    /// token. Useful to issue requests from several tasks in parallel.
    pub fn share(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            pending: self.pending.clone(),
            token: self.token.clone(),
            contracts: self.contracts.clone(),
        }
    }

    /// Sends Get(contract, id_request, token) and waits for Ack then Give.
    /// Returns the data.
    pub async fn get(&mut self, contract: &str, id_request: &str) -> Result<Vec<u8>> {
        // Register the answer channel BEFORE sending so nothing is lost.
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.pending
            .lock()
            .await
            .insert(id_request.to_string(), tx);
        self.tx
            .send(Message::Get {
                contract: contract.to_string(),
                id_request: id_request.to_string(),
                token: self.token.clone(),
            })
            .await
            .map_err(|_| Error::Protocol("connection to srv lost".into()))?;

        // Ack(id_request) then Give(id_request, data) or Error.
        let res = async {
            match next_reply(&mut rx, "Ack").await? {
                Message::Ack { id_request: id } if id == id_request => {}
                Message::Error { reason } => return Err(Error::Protocol(reason)),
                _ => return Err(Error::Protocol("Ack expected".into())),
            }
            match next_reply(&mut rx, "Give").await? {
                Message::Give { id_request: id, data } if id == id_request => Ok(data),
                Message::Error { reason } => Err(Error::Protocol(reason)),
                _ => Err(Error::Protocol("Give expected".into())),
            }
        }
        .await;

        self.pending.lock().await.remove(id_request);
        res
    }

    /// Sends Publish(contract, id_request, token) and waits for Ack then
    /// Published.
    pub async fn publish(&mut self, contract: &str, id_request: &str) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.pending
            .lock()
            .await
            .insert(id_request.to_string(), tx);
        self.tx
            .send(Message::Publish {
                contract: contract.to_string(),
                id_request: id_request.to_string(),
                token: self.token.clone(),
            })
            .await
            .map_err(|_| Error::Protocol("connection to srv lost".into()))?;

        let res = async {
            match next_reply(&mut rx, "Ack").await? {
                Message::Ack { id_request: id } if id == id_request => {}
                Message::Error { reason } => return Err(Error::Protocol(reason)),
                _ => return Err(Error::Protocol("Ack expected".into())),
            }
            match next_reply(&mut rx, "Published").await? {
                Message::Published { id_request: id } if id == id_request => Ok(()),
                Message::Error { reason } => Err(Error::Protocol(reason)),
                _ => Err(Error::Protocol("Published expected".into())),
            }
        }
        .await;

        self.pending.lock().await.remove(id_request);
        res
    }
}

// ---------------------------------------------------------------------------
// Integration tests: two apps, concurrency, multi-hop request routing.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // Server handler: answers directly for contract "weather", but for
    // contract "relay" it asks app2 for the data via Ctx::get_from.
    // ------------------------------------------------------------------
    struct RelayHandler;

    #[async_trait]
    impl Handler for RelayHandler {
        fn verify(&self, name: &str, password: &str) -> bool {
            // E.g. an argon2 hash check in production. Plain compare here.
            (name == "app1" || name == "app2") && password == "s3cret"
        }
        fn contracts(&self) -> Vec<String> {
            vec!["weather".into(), "stock".into(), "relay".into()]
        }
        async fn get(
            &self,
            ctx: &Ctx,
            _caller: &str,
            contract: &str,
            id_request: &str,
        ) -> std::result::Result<Vec<u8>, String> {
            match contract {
                "weather" => Ok(b"22C".to_vec()),
                // Multi-hop: srv itself queries app2 while serving app1.
                "relay" => ctx
                    .get_from("app2", "sensor", &format!("hop-{id_request}"))
                    .await
                    .map_err(|e| e.to_string()),
                _ => Err(format!("unknown contract {contract:?}")),
            }
        }
        async fn publish(
            &self,
            _ctx: &Ctx,
            _caller: &str,
            _contract: &str,
            _id_request: &str,
        ) -> std::result::Result<(), String> {
            Ok(())
        }
    }

    // ------------------------------------------------------------------
    // Client handler for app2: serves srv-initiated requests.
    // ------------------------------------------------------------------
    struct App2Handler;

    #[async_trait]
    impl ClientHandler for App2Handler {
        async fn get(
            &self,
            _contract: &str,
            id_request: &str,
        ) -> std::result::Result<Vec<u8>, String> {
            assert!(id_request.starts_with("hop-"), "unexpected id");
            Ok(format!("data-from-app2({id_request})").into_bytes())
        }
        async fn publish(
            &self,
            _contract: &str,
            _id_request: &str,
        ) -> std::result::Result<(), String> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn bad_credentials_are_rejected() {
        let sock = std::env::temp_dir().join("msglink-auth.sock");
        let server = Server::bind(&sock).await.unwrap();
        tokio::spawn(server.run(Arc::new(RelayHandler)));

        let err = Client::connect(&sock, "app1", "wrong", None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Auth(_)));
    }

    #[tokio::test]
    async fn server_not_running() {
        let err = Client::connect("/tmp/msglink-absent.sock", "app1", "x", None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotRunning(_)));
    }

    #[tokio::test]
    async fn single_client_get_and_publish() {
        let sock = std::env::temp_dir().join("msglink-single.sock");
        let server = Server::bind(&sock).await.unwrap();
        tokio::spawn(server.run(Arc::new(RelayHandler)));

        let (mut c, contracts) =
            Client::connect(&sock, "app1", "s3cret", None).await.unwrap();
        assert_eq!(
            contracts,
            vec!["weather".to_string(), "stock".to_string(), "relay".to_string()]
        );

        // Get -> Ack + Give
        assert_eq!(c.get("weather", "req-1").await.unwrap(), b"22C".to_vec());
        // Publish -> Ack + Published
        c.publish("weather", "req-2").await.unwrap();

        // Forged token: srv rejects it and closes the connection.
        c.token = Token("deadbeef".into());
        assert!(c.get("weather", "req-3").await.is_err());
    }

    // ------------------------------------------------------------------
    // Main scenario required: srv running, app1 connects, app2 connects.
    // app1 calls get() on contract "relay"; srv catches it and issues its
    // OWN get() to app2; app2 answers srv; srv returns that same data to
    // app1. srv handles both connections concurrently, without losing
    // messages. A parallel get() from app2 proves requests interleave.
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn two_apps_multi_hop_get() {
        let sock = std::env::temp_dir().join("msglink-multi.sock");
        let server = Server::bind(&sock).await.unwrap();
        tokio::spawn(server.run(Arc::new(RelayHandler)));

        // app1: pure client.
        let (mut app1, contracts) =
            Client::connect(&sock, "app1", "s3cret", None).await.unwrap();
        assert_eq!(contracts.len(), 3);

        // app2: client that also serves srv-initiated requests.
        let (mut app2, _) = Client::connect(
            &sock,
            "app2",
            "s3cret",
            Some(Arc::new(App2Handler)),
        )
        .await
        .unwrap();

        // Interleave a direct get() from app2 with the multi-hop get() from
        // app1: srv must serve both connections in parallel.
        let h1 = tokio::spawn(async move {
            app1.get("relay", "req-a").await.map(|d| d.len())
        });
        let h2 = tokio::spawn(async move {
            app2.get("weather", "req-b").await.map(|d| d.len())
        });

        // Multi-hop result: app1 receives app2's data through srv.
        let data = {
            // reconnect-free check: h1 returns the length; redo is not
            // possible after move, so assert lengths and check the id.
            h1.await.unwrap().unwrap()
        };
        let direct = h2.await.unwrap().unwrap();
        assert_eq!(direct, 3); // "22C"
        // "data-from-app2(hop-req-a)"
        assert_eq!(data, format!("data-from-app2(hop-req-a)").len());

        // Also verify the exact payload: rerun a full exchange.
        let (mut app1b, _) =
            Client::connect(&sock, "app1", "s3cret", None).await.unwrap();
        let got = app1b.get("relay", "req-c").await.unwrap();
        assert_eq!(got, format!("data-from-app2(hop-req-c)").into_bytes());

        // Publish flows still work on both connections.
        app1b.publish("relay", "pub-1").await.unwrap();
    }

    // ------------------------------------------------------------------
    // Concurrency: many parallel requests across both apps, no message
    // lost and no id_request mix-up.
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn parallel_requests_no_loss() {
        let sock = std::env::temp_dir().join("msglink-parallel.sock");
        let server = Server::bind(&sock).await.unwrap();
        tokio::spawn(server.run(Arc::new(RelayHandler)));

        let (app1, _) =
            Client::connect(&sock, "app1", "s3cret", None).await.unwrap();
        let (app2, _) = Client::connect(
            &sock,
            "app2",
            "s3cret",
            Some(Arc::new(App2Handler)),
        )
        .await
        .unwrap();

        // 16 parallel gets: half direct, half relayed through app2.
        let mut handles = Vec::new();
        for i in 0..16 {
            if i % 2 == 0 {
                let mut c = app1.share();
                handles.push(tokio::spawn(async move {
                    c.get("relay", &format!("p{i}")).await
                }));
            } else {
                let mut c = app2.share();
                handles.push(tokio::spawn(async move {
                    c.get("weather", &format!("p{i}")).await
                }));
            }
        }
        for (i, h) in handles.into_iter().enumerate() {
            let data = h.await.unwrap().unwrap();
            if i % 2 == 0 {
                assert_eq!(
                    data,
                    format!("data-from-app2(hop-p{i})").into_bytes()
                );
            } else {
                assert_eq!(data, b"22C".to_vec());
            }
        }
    }
}
