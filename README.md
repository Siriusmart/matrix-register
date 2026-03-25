A Discord bot to register users to private Matrix homeservers.

```bash
$ CONFIG=path/to/config.toml ./matrix-register
```

Config schema

```toml
discord-token = "BOT TOKEN"
homeserver-url = "https://matrix.example.com"
homeserver-domain = "example.com"
registration-token = "REGISTRATION TOKEN" # optional
guild-ids = [ 1234567, 1231423523] # list of discord servers to register the slash command to
```
