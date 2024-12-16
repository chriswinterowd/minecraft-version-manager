# mvm: Minecraft Version Manager

**mvm** is a simple CLI-based Minecraft version manager that allows users to manage Vanilla and Paper Minecraft servers effortlessly. It supports downloading, using, and managing specific versions of servers through intuitive subcommands.

---

## Features
- **Install** and **delete** Minecraft server versions (Vanilla or Paper).
- **Activate** a specific version for use.
- **Determine** the path of a specific or recent server version.
- Defaults to **Vanilla** server management unless the `--paper` flag is provided for Paper servers.

---

## Installation
1. **Clone the repository:**
   ```bash
   git clone https://github.com/chriswinterowd/minecraft-version-manager
   cd minecraft-version-manager
   ```

2. **Build the project:**
   ```bash
   cargo build --release
   ```

3. **Install the `mvm` binary globally:**
   Use a symlink to make the `mvm` executable available system-wide:
   ```bash
   sudo ln -s $(pwd)/target/release/mvm /usr/local/bin/mvm
   ```

4. **Verify the installation:**
   ```bash
   mvm --help
   ```
   This should display the CLI help message, confirming that `mvm` is installed correctly.

---

### Subcommands
| Command            | Description                                | 
|--------------------|--------------------------------------------|
| `install`          | Installs a specific or latest server.      | 
| `use`              | Activates a specified server version.      | 
| `uninstall`        | Removes a specific server version.         |
| `which`            | Determines the path of a specified version.|

### Flags
- `--paper` : Specifies that you want to manage **Paper** servers. If this flag is **not provided**, **Vanilla** servers are managed by default.

---

## Examples

1. **Install the latest Vanilla version:**
   ```bash
   mvm install
   ```

2. **Install the latest Paper version:**
   ```bash
   mvm install --paper
   ```

3. **Activate a specific Vanilla version:**
   ```bash
   mvm use 1.20.2
   ```

4. **Activate a specific Paper version:**
   ```bash
   mvm use 1.20.2 --paper
   ```

5. **Uninstall a specific Paper version:**
   ```bash
   mvm uninstall 1.20.2 --paper
   ```

6. **Find the path of the latest installed Vanilla version:**
   ```bash
   mvm which
   ```

---

## License
Refer to [LICENSE.md](https://github.com/chriswinterowd/minecraft-version-manager/blob/master/LICENSE.md).

---


