# **Minecraft Version Manager**

**mvm** is a CLI tool for managing Minecraft server versions. It allows users to install, use, and remove Minecraft server versions (both Vanilla and Paper), as well as determine the paths for existing server installations.

---

## **Features**
- **Install**: Download and install a specific or the latest server version.
- **Use**: Activate a specific version of a Minecraft server.
- **Uninstall**: Remove a server version.
- **Which**: Determine the path to a specific or recent server version.

MVM supports **Vanilla** and **Paper** server types, automatically handling version downloads and configuration.

---

## **Installation**

### Prerequisites
- **Rust**: Ensure you have Rust and Cargo installed.  
  [Get Rust here](https://www.rust-lang.org/tools/install).

### Build the Project
Run the following command to build the binary:
```bash
cargo build --release
```

### Install the Binary
Once built, the binary can be installed:
```bash
cargo install --path .
```

---

## **Usage**

### **Commands**
Below are the supported subcommands:

| Command               | Description                                   | Example                                  |
|-----------------------|-----------------------------------------------|------------------------------------------|
| `install`             | Installs a specific or latest server version. | `mvm install --paper`                    |
| `use`                 | Activates a specified server version.         | `mvm use 1.20 --paper`                   |
| `uninstall`           | Removes a specific server version.            | `mvm uninstall 1.20`                     |
| `which`               | Determines the path of a server version.      | `mvm which --paper`                      |

### **Global Flags**
- `--paper`: Specifies that the server type is Paper (defaults to Vanilla if omitted).

---

## **Examples**

### 1. **Install the Latest Paper Server**
```bash
mvm install --paper
```

### 2. **Activate a Vanilla Server Version**
```bash
mvm use 1.20.1
```

### 3. **Determine the Path for a Recent Server**
```bash
mvm which --paper
```

### 4. **Uninstall a Server Version**
```bash
mvm uninstall 1.20
```

---

## **Configuration**
MVM uses a configuration file `config.toml` stored in the `.mvm` directory under the home folder. It keeps track of the most recently used server versions.

---

## **License**
This project is licensed under the MIT License.
