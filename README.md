# ImHex Discord RPC

A Discord Rich Presence Client for ImHex, not reliant on the ImHex API. 
Windows only.

## Preview

![RPC](img/examplerpc.png?raw=true)

## Installing

- Download `imhex_rpc.exe` from [GitHub Releases](https://github.com/Atropa-Solanaceae/ImHex-Discord-RPC/releases/latest)
- Open the `shell:startup` folder with the windows run menu (`win + r`), and drag the executable there.
- Double click to run, or restart.

## Updating

- Exit ImHex_RPC

![exit](img/exit.png?raw=true)

- Download the version you want to update to
- Follow the instructions for installation.

Here are the instructions on how to build the ImHex Discord RPC from source:

## Building from Source

### Prerequisites

Before you can build the project, ensure you have the following installed:

1. **Rust**: You can install Rust using `rustup`. Follow the instructions on the [official Rust website](https://www.rust-lang.org/tools/install).
2. **Cargo**: This is included with the Rust installation.

### Cloning the Repository

1. Open your terminal or command prompt.
2. Navigate to the directory where you want to clone the repository.
3. Run the following command to clone the repository:

   ```bash
   git clone https://github.com/Atropa-Solanaceae/ImHex-Discord-RPC.git
   ```

4. Change to the project directory:

   ```bash
   cd ImHex-Discord-RPC
   ```

### Building the Project

1. To build the project in release mode, run the following command:

   ```bash
   cargo build --release
   ```

   This will compile the source and create an executable in the `target/release` directory.

## Support

Need help and can't get it to run correctly? Open an issue or contact me [here](https://solanaceae.xyz/).

## Sponsorship

If you like what I do, buy me a coffee so I can continue developing this tool and others!
[Ko-Fi](https://ko-fi.com/solanaceae)

---

## License

This project is licensed under the GNU General Public License v3.0 License. See the `LICENSE` file for more information.