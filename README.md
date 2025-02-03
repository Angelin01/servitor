# Servitor

**Servitor** is a *terrible idea* exposed as an extremely simple REST API, designed to control systemd services on your
system.
This project allows you to manage systemd services by starting, stopping, restarting, and checking their status. It
supports both session-level and system-level control through the D-Bus interface.

## Features

- **Start, stop, restart, reload and check the status of systemd services** via a REST API.
- Support for controlling **session** or **system** services using D-Bus.
- **Authentication** support via a pre-hashed token. Supports Argon2, Pbkdf2 and Scrypt in the PHC format (more below),
  in an attempt to make it a slightly less bad idea.
- **Allowlist** to restrict which services can be controlled, in a further attempt to make it a slightly less bad idea.

**It does not support HTTPS at all**, so you should probably run it behind a reverse proxy and TLS terminate it there.

## Configuration

The configuration is loaded exclusively from environment variables. By default, it will not run, as it has
authentication enabled, but requires a token. Below is a table of the available environment variables:

| Variable            | Description                                                                                                                                          |  Default Value   |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------|:----------------:|
| `SERV_BIND_ADDRESS` | The address and port for the API server to listen on. Defaults to localhost only, so change it to `0.0.0.0` if you want to expose it further.        | `127.0.0.1:8008` |
| `SERV_AUTH_ENABLED` | Whether authentication is enabled (`true` or `false`).                                                                                               |      `true`      |
| `SERV_AUTH_TOKEN`   | The token used for authentication when `SERV_AUTH_ENABLED` is `true`. See [Auth Token](#auth-token) bellow.                                          |      (None)      |
| `SERV_ALLOWLIST`    | A comma-separated list of allowed service names. If not supplied, allows all services.                                                               |      (None)      |
| `SERV_DBUS_SCOPE`   | The scope for D-Bus communication (`session` or `system`).                                                                                           |    `session`     |
| `SERV_LOG_LEVEL`    | The log level for the application (`trace`, `debug`, `info`, `warn`, `error`). See Rust's tracing and tracing-subscriber creates for advanced usage. |      `INFO`      |

## Usage

The REST API exposes the following endpoints for controlling systemd services and checking the server health:

### Service Endpoints

All service endpoint require a service name in the path. The full service name, including extension, must be provided.

- `POST /api/v1/services/{service}/start`  
  Starts the specified systemd service.  
  Example response:
  ```json
  {
    "service": "ssh-agent.service",
    "status": "starting"
  }
  ```

- `POST /api/v1/services/{service}/stop`  
  Stops the specified systemd service.  
  Example response:
  ```json
  {
    "service": "ssh-agent.service",
    "status": "stopping"
  }
  ```

- `POST /api/v1/services/{service}/restart`  
  Restarts the specified systemd service.  
  Example response:
  ```json
  {
    "service": "ssh-agent.service",
    "status": "restarting"
  }
  ```

- `POST /api/v1/services/{service}/reload`  
  Reload the specified systemd service.  
  Example response:
  ```json
  {
    "service": "ssh-agent.service",
    "status": "reloading"
  }
  ```

- `GET /api/v1/services/{service}/status`  
  Retrieves the status of the specified systemd service
  ```json
  {
    "service": "ssh-agent.service",
    "state": "active",
    "sub_state": "running",
    "since": "2025-02-02T22:40:34.252244Z"
  }
  ```

### Health Endpoint

- `GET /health`  
  Checks the health of the API server. It, unfortunately, does not actually check the underlying DBUS connection.

### Authentication

Authentication is handled as a standard Authorization Bearer token. All service endpoints are protected, but the health
endpoint is not.

If authentication is enabled, you must first [generate and configure a token](#auth-token), then you can simply pass it
in the Authorization header. For example, if your token is `banana`:

```shell
curl -H 'Authorization: Bearer banana' http://localhost:8008/
```

### Full Example

Here is a full example starting the service `ssh-agent.service`. checking it's status, and then stopping it, with an
authentication token `banana`, with the server running in localhost on port `8008`:

```shell
TOKEN=banana
curl -H "Authorization: Bearer $TOKEN" -X POST http://127.0.0.1:8008/api/v1/services/ssh-agent.service/start
curl -H "Authorization: Bearer $TOKEN" -X GET http://127.0.0.1:8008/api/v1/services/ssh-agent.service/status
curl -H "Authorization: Bearer $TOKEN" -X POST http://127.0.0.1:8008/api/v1/services/ssh-agent.service/stop
```

## Auth Token

The auth token, provided by the `SERV_AUTH_TOKEN` environment variable, is a pre-hashed PHC string. You can use Python's
`argon2-cffi` package to quickly generate a token:
```shell
pip install --user argon2-cffi
# Putting a single space before the command in shells like bash stops it from being recorded in history
# Obviously change `your_token` here to something more reasonable
 echo "your_token" | python -c 'import sys;from argon2 import PasswordHasher;p=PasswordHasher();print(p.hash(sys.stdin.readline().strip()))'
# $argon2id$v=19$m=65536,t=3,p=4$GZcsrX+VTsxFTtYKXW01XA$eSOI9RKxTqFfWwFvgnF9vKfiqERT2G9lBVlQ07qy1sk
```

Simply configure the token as the environment variable and you are good to go:
```shell
# Don't forget single quotes, or you'll have to escape all those $ signs
SERV_AUTH_TOKEN='$argon2id$v=19$m=65536,t=3,p=4$GZcsrX+VTsxFTtYKXW01XA$eSOI9RKxTqFfWwFvgnF9vKfiqERT2G9lBVlQ07qy1sk'
```

## Why is it a bad idea?

Well, let's be honest, exposing systemd controls over HTTP isn't exactly a "best practice" in security or reliability.
But hey, who needs security when you’ve got a nice, shiny API to mess with system services remotely, right? What could
possibly go wrong? Just imagine a world where your web server goes down, and someone starts it back up with a single
`POST` while you're on vacation.

In all seriousness, while it’s definitely fun to have the power to control your system at your fingertips, please don’t
use this in production or on anything that could potentially have a lot of angry users, this tool has NOT been audited,
and it provides no guarantees! Unless you’re into chaos, in which case, be my guest, and be sure to message me about it
because I'd love to hear it.
