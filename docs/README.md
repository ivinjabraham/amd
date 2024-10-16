# amFOSS Daemon

Discord bot used for the official amFOSS server for members. Built with [Serenity](https://www.github.com/serenity-rs/serenity) and [Poise](ttps://www.github.com/serenity-rs/poise).


## Feature Overview

### Command Handling

This bot uses `poise`, a command framework built on top of `serenity`. You can add commands in the `commands` module and get them registered using the `get_commands` function.

```rust
// Example command in src/commands.rs
#[poise::command(prefix_command)]
async fn amdctl(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("amD is up and running.").await?;
    Ok(())
}

pub fn get_commands() -> Vec<poise::Command<Data, Error>> {
    vec![amdctl()]
}
```

### Reaction Roles

amD supports automatic role assignment based on emoji reactions to specific messages. You can configure which messages and reactions trigger role assignemnt by modifying the `reaction_roles` Hashmap in the bot's `Data` struct.

## Setup

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) installed.
- A [Shuttle](https://www.shuttle.dev/) account and installation.
- A [Discord Bot Token](https://discord.com/developers/).

### Configuration

1. Clone the repository:
```
git clone https://github.com/amfoss/amd.git
cd amd
```

2. Create a `Secrets.toml` with your Discord token in it.
```
touch Secrets.toml
echo <YOUR TOKEN> >> Secrets.toml
```
3. Run the bot locally with `cargo shuttle run`. For instructions on how to deploy, refer [Shuttle docs](https://docs.shuttle.dev/getting-started/quick-start).

## Contributing

1. Fork the repository
2. Create your feature or fix branch (`git checkout -b feature/my-feature`).
3. Commit your changes and push to the branch.
4. Open your pull request to `main` or `develop`.

## License
This project is licensed under the GNU General Public License v3.0. See the LICENSE file for details.
