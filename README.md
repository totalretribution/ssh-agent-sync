# ssh-agent-sync

A tool to synchronize SSH keys from a running SSH agent to your SSH config file. This is useful when you are using an SSH agent that loads keys from a remote source, such as 1Password, and you want to use those keys with tools that read the SSH config file directly.

## The Problem

Some SSH servers have a limit on the number of authentication attempts (e.g., 6). If your SSH agent has more keys than this limit, the server may close the connection before the correct key is presented, leading to authentication failure. This tool solves this problem by creating specific `Host` entries in your SSH config file for each key, ensuring that only the correct key is used for each host.


## Features

-   Lists public keys available in the SSH agent, similar to `ssh-add -L`.
-   Syncs the keys from the SSH agent to your `~/.ssh/config` file.
-   Provides a command-line interface (`ssh-agent-sync`) for manual syncing.
-   Provides a graphical user interface (`ssh-agent-sync-gui`) that runs in the system tray for automatic syncing.

## Binaries

This project provides two binaries:

### `ssh-agent-sync`

This is a command-line tool that allows you to manually sync your SSH keys.

**Usage:**

```bash
# Print keys from the agent
ssh-agent-sync --print

# Sync keys from the agent to the SSH config
ssh-agent-sync --sync

# Force sync even if keys haven't changed
ssh-agent-sync --sync --force
```

### `ssh-agent-sync-gui`

This is a graphical tool that runs in your system tray. It can be configured to automatically sync your keys in the background.

## Build

You can build the binaries using Cargo:

```bash
# Build both the CLI and GUI
cargo build --release

# Or build them individually
# Build the CLI
cargo build --release --bin ssh-agent-sync

# Build the GUI
cargo build --release --bin ssh-agent-sync-gui
```

The binaries will be located in the `target/release` directory.

## See Also

- [Using Bitwarden's SSH Agent with WSL](https://blog.jkwmoore.dev/bitwarden-desktop-client-as-ssh-agent-with-wsl.html) - A guide on how to link Bitwarden's SSH agent to WSL, which can be used in conjunction with this tool.

## License

This project is licensed under the terms of the LICENSE file.
