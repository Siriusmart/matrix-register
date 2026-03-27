A Discord bot to register users to private Matrix homeservers.

```bash
$ CONFIG=path/to/config.toml ./matrix-register
```

Run with `UNREGISTER_COMMANDS=1` to unregister command on listed guild IDs.

Config schema

```toml
discord-token = "BOT TOKEN"
homeserver-url = "https://matrix.example.com"
homeserver-domain = "example.com"
registration-token = "REGISTRATION TOKEN" # optional
guild-ids = [ 1234567, 1231423523] # list of discord servers to register the slash command to
message = "Matrix is simple chat app powered by a decentralised protocol. If you join our homeserver, you can talk to anyone using Matrix - include people from other homeservers!\n\nYou can even see and send messages to any channel in this Discord server from Matrix, so you won't be missing out."
```
