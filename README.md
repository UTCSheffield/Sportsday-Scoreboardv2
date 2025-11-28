# Sportsday Scoreboard V2

This is gen 2 of the Scoreboard for the OLP Sports Day/Winter Games.

## Development

You should have experience with the following technologies before you start. Be warned!

- Rust
- Async Rust
- Stimulus JS
- Typescript
- Docker
- Nix
- SQL

To get a development environment, I suggest being on NixOS, and then running:

```
nix develop
```

This will setup all your dependencies and setup pre-commit and stuff.

You can then start the server and run tests with cargo/. To recompile the javascript (Something you will need to do the first time you start) run `bun scripts/build.ts`

You will also need the following environment variables defined in .env

```
GITHUB_OAUTH_CLIENT_ID=
GITHUB_OAUTH_CLIENT_SECRET=
```

These need to be set to a Github Oauth application with the callback of http://127.0.0.1:3000/oauth/callback

## Editing the Event Configuration

To Add/Change/Remove events, you can edit the config.yaml file. All the syntax is already in use in this file.
To make the server aware of the changes (to eg update for the new year) just change the version value.
