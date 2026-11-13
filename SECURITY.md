# Security policy

## Reporting a vulnerability

Please report vulnerabilities to the **Nodalia maintainers**, not in a public issue or pull request.
Use [GitHub private vulnerability reporting](https://github.com/getnodalia/forja/security/advisories/new)
when available. Include the affected version and platform, and a minimal description of how to
reproduce the problem. A small sample drawing helps, as long as it holds nothing private. Do not
share a working exploit, secrets, or sensitive files publicly. If private vulnerability reporting
is unavailable, contact the repository maintainers with a non-exploitable summary and request a
private channel before sharing reproduction steps or samples.

Maintainers read reports as they arrive, but there is no fixed response time yet.

## Supported versions

Only the latest release gets security fixes. Forja is pre-alpha and moves quickly, so please check
that a problem still exists there before reporting it.

## What counts

Forja treats these as untrusted, and a way past these defences is a vulnerability:

- **Drawing files.** Opening a DXF, DWG or other file must not run code, write outside the paths the user chose, or crash the app. Panics are bugs too, but memory exhaustion or a decoder abort on a crafted file is the more serious case.
- **AutoLISP in a drawing's folder.** A drawing must not run code that the user didn't choose to load.
- **The control channel.** It listens on loopback only and needs the session token from an owner-only file on every request. Another local account reaching it without that token is a vulnerability.
- **Temporary and autosave files.** Autosave files are unsaved drawings: another account must not be able to read them or redirect a write through them.
- **Updates.** A release must not be replaceable by someone other than the project.

## Known limits

These are known and being worked on, so they don't need a report:

- Agents connected through MCP or the control channel can read and write any path the user can. There are no workspace roots yet.
- A local process can make the control channel buffer up to 512 MiB before its token is checked. The total is capped, so this can slow the app but not exhaust memory without limit.
- The DWG worker process has no memory limit of its own yet, so a crafted file can use a lot of memory before it stops. It ends that one open or save, not the app.
- The web build has no autosave yet.

## What stays on your machine

Crash reports are written to the `crash-reports` folder in the settings folder and are never
uploaded. They hold the version, the platform, the panic message and a stack trace, and a message
can include a file path, so read a report before you attach it anywhere.
