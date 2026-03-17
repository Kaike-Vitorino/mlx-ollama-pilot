use std::io::{ErrorKind, SeekFrom};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::process::Command;
use tokio::time::timeout;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct OpenClawRuntimeConfig {
    pub node_command: String,
    pub cli_path: PathBuf,
    pub state_dir: PathBuf,
    pub gateway_token: String,
    pub session_key: String,
    pub timeout: Duration,
    pub gateway_log: PathBuf,
    pub error_log: PathBuf,
    pub sync_log: PathBuf,
}

#[derive(Debug, Clone)]
pub struct OpenClawRuntime {
    cfg: OpenClawRuntimeConfig,
}

impl OpenClawRuntime {
    pub fn new(cfg: OpenClawRuntimeConfig) -> Self {
        Self { cfg }
    }

    pub async fn status(&self) -> OpenClawStatusResponse {
        let mut response = OpenClawStatusResponse {
            available: false,
            cli_path: self.cfg.cli_path.display().to_string(),
            state_dir: self.cfg.state_dir.display().to_string(),
            session_key: self.cfg.session_key.clone(),
            gateway_log: self.cfg.gateway_log.display().to_string(),
            error_log: self.cfg.error_log.display().to_string(),
            sync_log: self.cfg.sync_log.display().to_string(),
            health: None,
            error: None,
        };

        if !self.cfg.cli_path.exists() {
            response.error = Some(format!(
                "openclaw cli nao encontrado em {}",
                self.cfg.cli_path.display()
            ));
            return response;
        }

        let health_timeout = Duration::from_secs(12);
        let args = vec![
            "gateway".to_string(),
            "call".to_string(),
            "--json".to_string(),
            "health".to_string(),
        ];

        match self.run_command_json(args, health_timeout).await {
            Ok(health) => {
                response.available = true;
                response.health = Some(health);
            }
            Err(error) => {
                response.error = Some(error.to_string());
            }
        }

        response
    }

    pub async fn read_logs(
        &self,
        query: OpenClawLogQuery,
    ) -> Result<OpenClawLogChunkResponse, OpenClawError> {
        let stream = query
            .stream
            .unwrap_or_else(|| "gateway".to_string())
            .to_lowercase();
        let path = self.resolve_log_path(&stream)?;
        let requested_cursor = query.cursor.unwrap_or(0);
        let max_bytes = query.max_bytes.unwrap_or(65536).clamp(1024, 262144);

        let metadata = match fs::metadata(&path).await {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(OpenClawLogChunkResponse {
                    stream,
                    path: path.display().to_string(),
                    exists: false,
                    cursor: requested_cursor,
                    next_cursor: requested_cursor,
                    file_size: 0,
                    truncated: false,
                    content: String::new(),
                });
            }
            Err(error) => {
                return Err(OpenClawError::Io {
                    context: format!("falha ao acessar {}", path.display()),
                    source: error.to_string(),
                });
            }
        };

        let file_size = metadata.len();
        let truncated = requested_cursor > file_size;
        let cursor = if truncated {
            0
        } else {
            requested_cursor.min(file_size)
        };
        let bytes_to_read = (file_size.saturating_sub(cursor) as usize).min(max_bytes);

        if bytes_to_read == 0 {
            return Ok(OpenClawLogChunkResponse {
                stream,
                path: path.display().to_string(),
                exists: true,
                cursor,
                next_cursor: cursor,
                file_size,
                truncated,
                content: String::new(),
            });
        }

        let mut file = fs::File::open(&path).await.map_err(|error| OpenClawError::Io {
            context: format!("falha ao abrir {}", path.display()),
            source: error.to_string(),
        })?;
        file.seek(SeekFrom::Start(cursor))
            .await
            .map_err(|error| OpenClawError::Io {
                context: format!("falha ao buscar offset em {}", path.display()),
                source: error.to_string(),
            })?;

        let mut buffer = vec![0_u8; bytes_to_read];
        file.read_exact(&mut buffer)
            .await
            .map_err(|error| OpenClawError::Io {
                context: format!("falha lendo {}", path.display()),
                source: error.to_string(),
            })?;

        let content = String::from_utf8_lossy(&buffer).to_string();
        let next_cursor = cursor + bytes_to_read as u64;

        Ok(OpenClawLogChunkResponse {
            stream,
            path: path.display().to_string(),
            exists: true,
            cursor,
            next_cursor,
            file_size,
            truncated,
            content,
        })
    }

    pub async fn chat(
        &self,
        request: OpenClawChatRequest,
    ) -> Result<OpenClawChatResponse, OpenClawError> {
        let message = request.message.trim();
        if message.is_empty() {
            return Err(OpenClawError::BadRequest(
                "message nao pode ser vazio".to_string(),
            ));
        }

        let idempotency_key = request
            .idempotency_key
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(generate_idempotency_key);

        let session_key = request
            .session_key
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| self.cfg.session_key.clone());

        let timeout_ms = request
            .timeout_ms
            .unwrap_or(self.cfg.timeout.as_millis() as u64)
            .clamp(1000, 900000);
        let timeout_limit = Duration::from_millis(timeout_ms + 2000);

        let params = json!({
            "message": message,
            "idempotencyKey": idempotency_key,
            "sessionKey": session_key,
        });
        let params_json = serde_json::to_string(&params).map_err(|error| OpenClawError::Parse {
            details: format!("falha serializando params: {error}"),
        })?;

        let args = vec![
            "gateway".to_string(),
            "call".to_string(),
            "agent".to_string(),
            "--expect-final".to_string(),
            "--json".to_string(),
            "--timeout".to_string(),
            timeout_ms.to_string(),
            "--params".to_string(),
            params_json,
        ];

        let response = self.run_command_json(args, timeout_limit).await?;
        Ok(normalize_chat_response(response))
    }

    fn resolve_log_path(&self, stream: &str) -> Result<PathBuf, OpenClawError> {
        let path = match stream {
            "gateway" => self.cfg.gateway_log.clone(),
            "error" => self.cfg.error_log.clone(),
            "sync" => self.cfg.sync_log.clone(),
            _ => {
                return Err(OpenClawError::BadRequest(
                    "stream invalido: use gateway, error ou sync".to_string(),
                ));
            }
        };

        Ok(path)
    }

    async fn run_command_json(
        &self,
        args: Vec<String>,
        timeout_limit: Duration,
    ) -> Result<Value, OpenClawError> {
        let mut command = Command::new(&self.cfg.node_command);
        command
            .arg(&self.cfg.cli_path)
            .args(&args)
            .env("OPENCLAW_STATE_DIR", &self.cfg.state_dir)
            .env("OPENCLAW_GATEWAY_TOKEN", &self.cfg.gateway_token)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let command_preview = format!(
            "{} {} {}",
            self.cfg.node_command,
            self.cfg.cli_path.display(),
            args.join(" ")
        );

        let output = timeout(timeout_limit, command.output())
            .await
            .map_err(|_| OpenClawError::Timeout {
                seconds: timeout_limit.as_secs().max(1),
            })?
            .map_err(|error| OpenClawError::Io {
                context: format!("falha executando {command_preview}"),
                source: error.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !output.status.success() {
            return Err(OpenClawError::CommandFailed {
                command: command_preview,
                stderr: if stderr.is_empty() {
                    "sem stderr".to_string()
                } else {
                    stderr
                },
            });
        }

        parse_json_output(&stdout).map_err(|details| OpenClawError::Parse {
            details: format!("{details}; stderr: {stderr}"),
        })
    }
}

#[derive(Debug)]
pub enum OpenClawError {
    BadRequest(String),
    Io { context: String, source: String },
    CommandFailed { command: String, stderr: String },
    Parse { details: String },
    Timeout { seconds: u64 },
}

impl std::fmt::Display for OpenClawError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenClawError::BadRequest(message) => write!(formatter, "{message}"),
            OpenClawError::Io { context, source } => write!(formatter, "{context}: {source}"),
            OpenClawError::CommandFailed { command, stderr } => {
                write!(formatter, "comando falhou ({command}): {stderr}")
            }
            OpenClawError::Parse { details } => write!(formatter, "{details}"),
            OpenClawError::Timeout { seconds } => {
                write!(formatter, "operacao expirou apos {seconds}s")
            }
        }
    }
}

impl std::error::Error for OpenClawError {}

#[derive(Debug, Deserialize)]
pub struct OpenClawLogQuery {
    pub stream: Option<String>,
    pub cursor: Option<u64>,
    pub max_bytes: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct OpenClawLogChunkResponse {
    pub stream: String,
    pub path: String,
    pub exists: bool,
    pub cursor: u64,
    pub next_cursor: u64,
    pub file_size: u64,
    pub truncated: bool,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct OpenClawStatusResponse {
    pub available: bool,
    pub cli_path: String,
    pub state_dir: String,
    pub session_key: String,
    pub gateway_log: String,
    pub error_log: String,
    pub sync_log: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OpenClawChatRequest {
    pub message: String,
    pub session_key: Option<String>,
    pub idempotency_key: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct OpenClawChatResponse {
    pub run_id: Option<String>,
    pub status: Option<String>,
    pub summary: Option<String>,
    pub reply: String,
    pub payloads: Vec<String>,
    pub duration_ms: Option<u64>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub usage: Option<OpenClawUsage>,
    pub skills: Vec<String>,
    pub tools: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct OpenClawUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

fn generate_idempotency_key() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or(0);
    let counter = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("mlx-pilot-openclaw-{millis}-{counter}")
}

fn parse_json_output(raw: &str) -> Result<Value, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("stdout vazio".to_string());
    }

    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        return Ok(value);
    }

    if let Some(extracted) = extract_last_json_object(trimmed) {
        if let Ok(value) = serde_json::from_str::<Value>(extracted) {
            return Ok(value);
        }
    }

    Err("retorno nao e JSON valido".to_string())
}

fn extract_last_json_object(raw: &str) -> Option<&str> {
    for (index, ch) in raw.char_indices().rev() {
        if ch != '{' {
            continue;
        }
        let candidate = raw.get(index..)?;
        if serde_json::from_str::<Value>(candidate).is_ok() {
            return Some(candidate);
        }
    }

    None
}

fn normalize_chat_response(raw: Value) -> OpenClawChatResponse {
    let run_id = get_string(&raw, "/runId");
    let status = get_string(&raw, "/status");
    let summary = get_string(&raw, "/summary");
    let duration_ms = get_u64(&raw, "/result/meta/durationMs");
    let provider = get_string(&raw, "/result/meta/agentMeta/provider")
        .or_else(|| get_string(&raw, "/result/meta/systemPromptReport/provider"));
    let model = get_string(&raw, "/result/meta/agentMeta/model")
        .or_else(|| get_string(&raw, "/result/meta/systemPromptReport/model"));

    let payloads = raw
        .pointer("/result/payloads")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry.get("text").and_then(Value::as_str))
                .map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let reply = if payloads.is_empty() {
        String::new()
    } else {
        payloads.join("\n\n")
    };

    let usage = build_usage(
        raw.pointer("/result/meta/agentMeta/usage")
            .unwrap_or(&Value::Null),
    );

    let skills = list_entry_names(&raw, "/result/meta/systemPromptReport/skills/entries");
    let tools = list_entry_names(&raw, "/result/meta/systemPromptReport/tools/entries");

    OpenClawChatResponse {
        run_id,
        status,
        summary,
        reply,
        payloads,
        duration_ms,
        provider,
        model,
        usage,
        skills,
        tools,
    }
}

fn build_usage(value: &Value) -> Option<OpenClawUsage> {
    if !value.is_object() {
        return None;
    }

    let usage = OpenClawUsage {
        input: value.get("input").and_then(Value::as_u64),
        output: value.get("output").and_then(Value::as_u64),
        cache_read: value.get("cacheRead").and_then(Value::as_u64),
        cache_write: value.get("cacheWrite").and_then(Value::as_u64),
        total: value.get("total").and_then(Value::as_u64),
    };

    if usage.input.is_none()
        && usage.output.is_none()
        && usage.cache_read.is_none()
        && usage.cache_write.is_none()
        && usage.total.is_none()
    {
        return None;
    }

    Some(usage)
}

fn list_entry_names(root: &Value, pointer: &str) -> Vec<String> {
    let mut names = Vec::new();

    if let Some(entries) = root.pointer(pointer).and_then(Value::as_array) {
        for entry in entries {
            if let Some(name) = entry.get("name").and_then(Value::as_str) {
                let normalized = name.trim();
                if !normalized.is_empty() && !names.iter().any(|value| value == normalized) {
                    names.push(normalized.to_string());
                }
            }
        }
    }

    names
}

fn get_string(root: &Value, pointer: &str) -> Option<String> {
    root.pointer(pointer)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn get_u64(root: &Value, pointer: &str) -> Option<u64> {
    root.pointer(pointer).and_then(Value::as_u64)
}
