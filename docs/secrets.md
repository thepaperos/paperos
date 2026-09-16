# PaperOS Secret Management

## Secure Extension and Configuration Architecture

## 1. Objective

Design a secure, extensible secret-management system for PaperOS.

The system must:

* keep secrets separate from normal configuration
* prevent extensions from accessing arbitrary secrets
* use capability-based authorization
* support OS-native credential stores
* support headless/server environments
* avoid exposing secrets through logs, events, command history, telemetry, crash reports, or AI context
* allow extensions to declare the secrets they require
* provide a stable API independent of the underlying secret backend
* minimize the amount of time secrets exist in memory
* make insecure secret handling difficult by default

The core principle is:

> Extensions never own the secret store. PaperOS Core controls secret access and grants access only through explicit capabilities.

---

# 2. Architecture

Use the following architecture:

```text
                         PaperOS
                            │
                    ┌───────▼────────┐
                    │   Secret API   │
                    │   Core Layer   │
                    └───────┬────────┘
                            │
                    Capability Check
                            │
                    ┌───────▼────────┐
                    │ Secret Manager │
                    └───────┬────────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
              ▼             ▼             ▼
        OS Keychain      Vault/Agent    Environment
              │             │             │
              └─────────────┼─────────────┘
                            │
                       Secret Handle
                            │
                            ▼
                        Extension
```

The extension must not know which backend is being used.

---

# 3. Separate Configuration and Secrets

Normal configuration must never contain plaintext credentials.

Allowed:

```toml
[github]
username = "azhar"
default_repository = "paperos"
```

Not allowed:

```toml
[github]
token = "ghp_xxxxxxxxx"
```

Instead:

```toml
[github]
account = "github"
```

The extension can declare:

```toml
[[secrets]]
name = "github.token"
```

The secret itself lives in the Secret Manager.

---

# 4. Secret Identity

Use structured secret identifiers instead of arbitrary strings.

Example:

```rust
pub struct SecretId {
    pub namespace: String,
    pub name: String,
}
```

Examples:

```text
github/token
github/app-token
gitlab/token
openai/api-key
anthropic/api-key
database/password
ssh/github
```

Prefer:

```text
github/token
```

over:

```text
some_random_secret_123
```

The namespace should normally correspond to the extension or service.

---

# 5. Extension Manifest

Extensions must explicitly declare secret requirements.

Example:

```toml
id = "paperos.github"

[capabilities]
secrets = [
    "github/token"
]

[[secrets]]
id = "github/token"
description = "GitHub authentication token"
```

The extension manager validates this during installation/loading.

Do not allow an extension to silently discover arbitrary secrets.

---

# 6. Capability-Based Authorization

Secret access must be capability-controlled.

Bad:

```rust
secret_manager.get("github/token")
```

Good:

```text
Extension
    ↓
CapabilitySet
    ↓
SecretManager
    ↓
authorized secret
```

The capability should be checked every time access is requested.

Never rely only on the manifest.

The manifest declares what the extension wants.

The granted capability determines what it actually receives.

---

# 7. Capability Granularity

Avoid a broad permission such as:

```text
secrets.read = true
```

Prefer:

```text
secrets.read.github/token
```

or:

```text
secrets.github.token
```

Potential future capabilities:

```text
secrets.read.github/token
secrets.write.github/token
secrets.delete.github/token
```

By default:

```text
read = denied
write = denied
delete = denied
```

Extensions receive only the minimum required permissions.

---

# 8. Secret Store Trait

Create an abstraction independent of the backend.

Example:

```rust
pub trait SecretStore: Send + Sync {
    fn get(&self, id: &SecretId) -> Result<Secret, SecretError>;

    fn set(
        &self,
        id: &SecretId,
        secret: Secret,
    ) -> Result<(), SecretError>;

    fn delete(
        &self,
        id: &SecretId,
    ) -> Result<(), SecretError>;

    fn exists(
        &self,
        id: &SecretId,
    ) -> Result<bool, SecretError>;
}
```

Do not expose backend-specific types to extensions.

---

# 9. Backend Implementations

Implement backends behind the trait.

Initial architecture:

```text
SecretStore
├── OsCredentialStore
├── EnvironmentSecretStore
├── FileVaultStore
└── ExternalSecretStore
```

Recommended priority:

### Desktop

Use the operating system credential store.

```text
Linux
macOS
Windows
```

### Headless/server

Support:

```text
environment variables
```

and later:

```text
external secret providers
```

### Development

A local encrypted vault can be provided for development/testing.

Do not make plaintext files the normal production backend.

---

# 10. Never Store Plaintext Secrets in Configuration

Do not implement:

```toml
api_key = "secret"
password = "secret"
token = "secret"
```

as a supported configuration mechanism.

If configuration needs to reference a secret, use a reference:

```toml
api_key = "secret://openai/api-key"
```

However, the preferred approach is for the extension API to request the secret directly.

Avoid making every extension implement its own `secret://` parser.

---

# 11. Secret Object

Do not represent secrets as ordinary `String` values throughout the system.

Create a dedicated type:

```rust
pub struct Secret {
    // private fields
}
```

The API should intentionally make accidental copying difficult.

Prefer:

```rust
secret.expose(|value| {
    // use secret
});
```

rather than encouraging:

```rust
let token: String = secret.to_string();
```

Avoid implementing:

```rust
Display
Debug
Serialize
```

for the secret value.

If `Debug` must be implemented, output:

```text
Secret(REDACTED)
```

never the actual value.

---

# 12. Secret Lifetime

Minimize the time secrets exist in memory.

Prefer:

```rust
secret.with_value(|value| {
    perform_authenticated_operation(value)
});
```

over:

```rust
let token = secret.to_string();

do_many_things(token);
```

The API should encourage short-lived access.

Do not retain secrets inside long-lived actors unless absolutely necessary.

If a service needs authentication repeatedly, prefer a dedicated credential/session abstraction rather than passing raw secrets throughout the system.

---

# 13. Secret Handles

Consider introducing:

```rust
SecretHandle
```

for long-lived references.

Example:

```text
SecretHandle
    ↓
SecretManager
    ↓
credential backend
```

The handle identifies a secret but does not itself contain the secret.

This allows:

```text
GitActor
    ↓
GitHubCredentialHandle
    ↓
SecretManager
```

instead of:

```text
GitActor
    ↓
raw token stored permanently in actor state
```

---

# 14. Never Put Secrets in Events

This is a strict architectural rule.

Never:

```rust
Event::Authenticated {
    token,
}
```

Never emit:

```rust
Event::SecretLoaded {
    value,
}
```

Instead:

```rust
Event::Authenticated {
    account: "github",
}
```

Events may contain:

```text
secret identifier
account identifier
credential status
authentication result
```

but never the secret value.

---

# 15. Never Put Secrets in Commands

Avoid:

```rust
Command::GitClone {
    url,
    username,
    password,
}
```

Prefer:

```rust
Command::GitClone {
    repository,
    credential: CredentialRef,
}
```

The Git service resolves the credential through the Secret Manager.

---

# 16. Logging Protection

Secrets must never appear in:

```text
logs
tracing
error messages
events
metrics
telemetry
panic messages
```

Add redaction support.

Example:

```rust
tracing::info!(
    account = %account,
    credential = ?Redacted(secret),
    "authentication configured"
);
```

Output:

```text
credential=[REDACTED]
```

Add tests specifically checking that known test secrets cannot appear in logs.

---

# 17. Error Handling

Never return:

```text
authentication failed using token ghp_xxxxx
```

Instead:

```text
authentication failed
```

or:

```text
authentication failed for github account "work"
```

Backend errors containing credentials must be sanitized before crossing the extension boundary.

---

# 18. Command History

Do not allow secret values in command history.

Bad:

```text
git.login --token ghp_xxxxx
```

Good:

```text
github.authenticate
```

If a command accepts sensitive arguments, mark the arguments:

```rust
ArgumentMetadata {
    sensitive: true,
}
```

Sensitive command arguments must not enter:

```text
command history
logs
analytics
undo history
AI context
```

---

# 19. AI Security

AI integration requires special handling.

Never automatically expose the entire Secret Manager to AI.

Bad:

```text
AI → all secrets
```

Instead:

```text
AI
 ↓
Git service
 ↓
Credential capability
 ↓
Secret Manager
```

The AI should normally interact with a service such as:

```text
git.push
git.fetch
github.create_issue
```

rather than receiving:

```text
github/token
```

directly.

This follows the principle:

> Give AI the capability to perform an operation, not unnecessary access to the credential that enables it.

---

# 20. Lua Security

Lua extensions must also be capability-controlled.

Do not expose:

```lua
secret.get_all()
```

or:

```lua
secret.list()
```

to arbitrary scripts.

Prefer:

```lua
secret.with("github/token", function(secret)
    -- controlled operation
end)
```

and require:

```text
secrets.read.github/token
```

capability.

For untrusted Lua, consider not exposing raw secret values at all.

Instead provide service APIs:

```lua
github.create_issue(...)
github.create_pull_request(...)
```

---

# 21. Extension-Owned Secrets

An extension may define its own logical secret namespace:

```text
extension.example/api-key
extension.example/password
```

but the physical storage remains controlled by PaperOS.

Do not allow an extension to create its own:

```text
~/.paperos/extensions/example/secrets.json
```

as a normal mechanism.

This prevents every extension from inventing a different security model.

---

# 22. Workspace vs Global Secrets

Support different scopes.

```text
Global
Workspace
Account/Profile
Extension
```

Example:

```text
global/github/token
workspace/project-a/github/token
```

But workspace-specific secrets require additional protection.

Never silently allow a project to request credentials belonging to another project.

For untrusted workspaces:

```text
secret access = denied by default
```

This is especially important for repositories containing malicious configuration or extensions.

---

# 23. Trusted Workspace Model

Introduce workspace trust.

```text
Workspace
├── Trusted
└── Restricted
```

Restricted workspace:

```text
filesystem = restricted
process = restricted
network = restricted
secrets = restricted
extensions = restricted
```

Trusted workspace:

```text
permissions depend on extension capabilities
```

Do not treat opening a directory as permission to access the user's credentials.

---

# 24. Secret Prompt UI

The UI should provide a generic secret-management interface.

Example:

```text
GitHub Authentication

Account: Work GitHub

Token: **************

[ Save ]
[ Remove ]
[ Test ]
```

The UI communicates with the Secret Manager through commands.

It must never access the secret backend directly.

---

# 25. Secret Commands

Core commands may include:

```text
secret.set
secret.delete
secret.exists
secret.list_metadata
secret.test
```

Avoid a general-purpose:

```text
secret.get
```

being available to every UI component.

Secret retrieval should primarily occur inside authorized service/extension contexts.

---

# 26. Clipboard Protection

Copying secrets to clipboard should be an explicit operation.

If supported:

```text
secret.copy_to_clipboard
```

must:

* require explicit user action
* show confirmation where appropriate
* optionally clear clipboard after a timeout
* never log the value

Do not automatically copy secrets.

---

# 27. Environment Variables

Environment variables are useful for headless operation:

```text
PAPEROS_SECRET_GITHUB_TOKEN
```

but should not automatically become visible to all extensions.

Instead:

```text
Environment
    ↓
Secret Provider
    ↓
Capability Check
    ↓
Authorized Extension
```

Do not implement:

```rust
std::env::vars()
```

as a secret API for extensions.

---

# 28. Process Spawning

Be careful when secrets are used with child processes.

Avoid:

```rust
Command::new("git")
    .env("TOKEN", secret)
```

unless necessary.

Environment variables can leak through:

```text
process inspection
child processes
debugging
crash dumps
process listings
```

Prefer:

```text
credential helpers
stdin
temporary controlled channels
OS credential integration
```

where practical.

---

# 29. Secret Redaction Infrastructure

Build a reusable redaction system in the core.

Example:

```rust
Redacted<T>
```

and:

```rust
Sensitive<T>
```

These types should prevent accidental formatting.

Use them throughout:

```text
logging
errors
commands
events
telemetry
AI context
```

---

# 30. Testing Requirements

Create security tests for:

### Access control

```text
extension without capability → denied
extension with capability → allowed
wrong secret namespace → denied
```

### Logging

Verify:

```text
known secret
```

never appears in logs.

### Events

Verify secrets cannot be serialized into events.

### Commands

Verify sensitive arguments are not stored in history.

### AI

Verify secret values cannot accidentally enter AI context.

### Lua

Verify unauthorized scripts cannot access secrets.

### Workspace

Verify restricted workspaces cannot access protected credentials.

### Backend

Test:

```text
OS store
environment store
encrypted store
```

independently.

---

# 31. Security Invariants

These should become permanent PaperOS architectural rules.

### Invariant 1

> No extension receives unrestricted access to the Secret Manager.

### Invariant 2

> Secrets never appear in normal configuration files.

### Invariant 3

> Secret values never appear in events.

### Invariant 4

> Secret values never appear in logs or telemetry.

### Invariant 5

> Secret values are never serialized through the UI protocol.

### Invariant 6

> Secret access requires explicit capability authorization.

### Invariant 7

> Restricted workspaces cannot access global credentials by default.

### Invariant 8

> AI does not receive unrestricted secret access.

### Invariant 9

> Lua does not receive unrestricted secret access.

### Invariant 10

> Backend implementation details never leak into extension APIs.

---

# 32. Implementation Order

Implement vertically rather than building every backend first.

### Phase 1 — Core API

Implement:

```text
SecretId
Secret
SecretHandle
SecretStore
SecretManager
SecretError
```

### Phase 2 — Capability Integration

Implement:

```text
secrets.read.*
secrets.write.*
secrets.delete.*
```

and enforce them through the extension manager.

### Phase 3 — Development Backend

Implement an encrypted local development store.

Use it for automated tests.

### Phase 4 — Environment Backend

Implement environment-backed secrets for headless environments.

### Phase 5 — OS Credential Backend

Implement native desktop credential storage.

Keep this behind:

```rust
SecretStore
```

### Phase 6 — Extension Integration

Integrate with:

```text
Git
GitHub
GitLab
AI
Terminal
Database
Cloud
```

one extension at a time.

### Phase 7 — UI

Build:

```text
Secret Manager
Authentication
Credential Status
Remove Credential
Test Credential
```

as normal PaperOS commands/views.

### Phase 8 — Security Hardening

Audit:

```text
logs
events
commands
Lua
AI
process spawning
workspace trust
serialization
crash handling
telemetry
```

---

# 33. Suggested Rust Module Structure

```text
paperos-core/
└── src/
    ├── secrets/
    │   ├── mod.rs
    │   ├── id.rs
    │   ├── secret.rs
    │   ├── handle.rs
    │   ├── manager.rs
    │   ├── capability.rs
    │   ├── error.rs
    │   ├── store.rs
    │   ├── environment.rs
    │   └── redaction.rs
    │
    ├── commands/
    ├── events/
    ├── capabilities/
    └── extensions/
```

Backends should ideally live separately:

```text
paperos-secret-os
paperos-secret-env
paperos-secret-vault
```

This keeps the core abstraction stable.

---

# 34. Important Design Decision

Do not make the Secret Manager a Runact actor just because PaperOS uses Runact.

The recommended design is:

```text
SecretManager
    ↓
synchronous lightweight API
    ↓
backend
```

Use actors when the operation genuinely benefits from:

* isolation
* asynchronous I/O
* lifecycle management
* concurrency
* supervision

The command dispatcher and secret authorization path should remain lightweight.

---

# 35. Final Architecture

The final design should look like:

```text
                       ┌───────────────┐
                       │    PaperOS    │
                       │     Core      │
                       └───────┬───────┘
                               │
                    ┌──────────▼──────────┐
                    │   Secret Manager    │
                    └──────────┬──────────┘
                               │
                       Capability Check
                               │
                ┌──────────────┼──────────────┐
                │              │              │
                ▼              ▼              ▼
             OS Store       Env Store      Vault
                │              │              │
                └──────────────┼──────────────┘
                               │
                         Secret Handle
                               │
             ┌─────────────────┼─────────────────┐
             ▼                 ▼                 ▼
           Git               AI                Lua
             │                 │                 │
       authorized        service-level       authorized
        operation         operation            access
```

## Final Principle

PaperOS should treat secrets as a **security-sensitive capability**, not as configuration data.

The ideal flow is:

```text
User
 ↓
Capability Grant
 ↓
Extension
 ↓
Service
 ↓
Secret Manager
 ↓
Credential Backend
```

and never:

```text
Extension
 ↓
read ~/.paperos/secrets.json
 ↓
get everything
```

The architecture should make the secure path the easiest path for extension developers.