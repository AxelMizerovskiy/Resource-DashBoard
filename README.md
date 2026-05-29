# rdb

`rdb` is a CLI resource monitor written in Rust. It automatically adapts to your terminal size, but make sure your window is at least 21 lines tall so everything renders correctly.

## Installation

Make sure you have Rust installed, then clone the repo and install it globally using Cargo:

```bash
git clone https://github.com/AxelMizerovskiy/Resource-DashBoard
cd rdb
cargo install --path .

```

## Usage

Once installed, you can launch the monitor from anywhere by typing:

* **Default run:**
```bash
rdb

```


* **Custom polling timeout (in seconds):**
```bash
rdb --timeout 5
# or
rdb -t 5

```



## Future Updates

* Add options to only show specific metrics (e.g., hiding disk usage or network I/O if you only want to see CPU/RAM).
