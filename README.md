# ssh-agent-sync

A tool to synchronize SSH keys from a running SSH agent to your SSH config file. This is useful when you are using an SSH agent that loads keys from a remote source, such as 1Password, and you want to use those keys with tools that read the SSH config file directly.

## The Problem

Some SSH servers have a limit on the number of authentication attempts (e.g., 6). If your SSH agent has more keys than this limit, the server may close the connection before the correct key is presented, leading to authentication failure. This tool solves this problem by creating specific `Host` entries in your SSH config file for each key, ensuring that only the correct key is used for each host.


## Features

-   Lists public keys available in the SSH agent, similar to `ssh-add -L`.
-   Syncs the keys from the SSH agent to your `~/.ssh/config` file.
-   Provides a command-line interface (`ssh-agent-sync`) for manual syncing.
-   Provides a graphical user interface (`ssh-agent-sync-gui`) that runs in the system tray for automatic syncing.

## Key Comment Format

For `ssh-agent-sync` to correctly identify and create `Host` entries, the comment associated with each SSH key in the agent must follow a specific format. The tool supports two formats for the key comment:

> [!NOTE]
> For Bitwarden users, the "name" of the SSH key is used as the comment.

### Simple `user@host`

The most basic format is `user@host`, with no spaces.

**Example:** `dev@githost.com`

This will generate a single `Host` entry in your SSH configuration:

```ssh-config
Host githost.com
    User dev
    IdentityFile /path/to/your/keys/githost_com.pub
    IdentitiesOnly yes
```

### Nickname `<user@host>`

You can also assign a nickname to a key, which is useful for creating aliases for hosts.

**Example:** `My-Server <dev@githost.com>`

This format will generate two `Host` entries: one for the nickname and one for the actual hostname. Any spaces in the nickname will be replaced with underscores.

```ssh-config
Host My-Server
    HostName githost.com
    User dev
    IdentityFile /path/to/your/keys/My-Server.pub
    IdentitiesOnly yes

Host githost.com
    User dev
    IdentityFile /path/to/your/keys/My-Server.pub
    IdentitiesOnly yes
```

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

## Dependencies

To build this project, you will need to have the Rust toolchain installed. You can install it from [rustup.rs](https://rustup.rs/).

### Linux

On Debian-based distributions, you will need to install the following packages:

```bash
sudo apt-get install -y \
    pkg-config \
    libglib2.0-dev \
    libssh2-1-dev \
    libdbus-1-dev \
    libxkbcommon-dev \
    libwayland-dev \
    libx11-dev \
    libgtk-3-dev \
    libxdo-dev
```

### Windows


For Windows, you will need the MSVC build tools. While the Visual Studio Installer can be used, it is often more reliable to install them using `winget` with the following command:

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.26100 --quiet --wait --norestart --nocache"
```

This method tends to work better than the official Rust installer for setting up the required build environment.

## See Also

- [Using Bitwarden's SSH Agent with WSL](https://blog.jkwmoore.dev/bitwarden-desktop-client-as-ssh-agent-with-wsl.html) - A guide on how to link Bitwarden's SSH agent to WSL, which can be used in conjunction with this tool.

## License

This project is licensed under the terms of the LICENSE file.
