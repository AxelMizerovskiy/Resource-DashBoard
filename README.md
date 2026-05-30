# rdb

rdb is a CLI resource monitor. It adapts to your terminal size, but it should be at least 21 lines in length so the interface renders properly.

## Installation

**Standalone Binary (No Rust required)**
If you are on Linux, you don't need to install the Rust toolchain. You can just download the compiled binary and move it into your system's PATH.

1. Download the latest `rdb` binary from the Releases tab.
2. Open your terminal in the folder where you downloaded it.
3. Make the file executable and move it to `/usr/local/bin` so your system recognizes the command globally:
```
   chmod +x rdb
   sudo mv rdb /usr/local/bin/
```
**Building from source**
If you already have Rust installed, you can clone the repository and compile it yourself:
```
   git clone https://github.com/AxelMizerovskiy/Resource-DashBoard
   cd rdb
   cargo install --path .
```
## Usage

After installation, you can use it by typing:

- `rdb` for the default call
- `rdb --timeout 5` or `rdb -t 5` for a specific timeout period (in seconds)
- `q` to close

## Future Updates
Future updates will give the user the option to only show specific metrics instead of the entire dashboard.
