<div align="center" style="margin-bottom: 20px">
    <h1>Hexd</h1>
    <p><em>A convenient, configurable, and dependency-free hexdump library for Rust.</em></p>
</div>

<div align="center" style="margin-bottom: 20px">
    <a href="https://crates.io/crates/hxd">
        <img alt="Crates.io Version" src="https://img.shields.io/crates/v/hxd?logo=rust" height="20">
    </a>
    <a href="https://docs.rs/hxd/latest/hxd/">
        <img alt="docs.rs" src="https://img.shields.io/docsrs/hxd?logo=docs.rs" height="20">
    </a>
    <a href="https://crates.io/crates/hxd">
        <img alt="Crates.io Size" src="https://img.shields.io/crates/size/hxd">
    </a>
    <a href="https://github.com/benjdod/hexd"><img
        alt="github"
        src="https://img.shields.io/badge/github-hexd-228b22?logo=github"
        height="20"
    /></a>
    <a href="https://github.com/benjdod/hexd/blob/master/LICENSE.txt">
        <img alt="docs.rs" src="https://img.shields.io/crates/l/hxd" height="20">
    </a>
</div>

Hexd is a debugging library that allows you to easily write hexdumps. It aims to be lightweight, easily within reach, and usable by default.

```rust
use hxd::AsHexd;
let s = "A convenient, configurable, and dependency-free hexdump library for Rust.";
s.hexd().dump();
```

```text
00000000: 4120 636F 6E76 656E 6965 6E74 2C20 636F |A convenient, co|
00000010: 6E66 6967 7572 6162 6C65 2C20 616E 6420 |nfigurable, and |
00000020: 6465 7065 6E64 656E 6379 2D66 7265 6520 |dependency-free |
00000030: 6865 7864 756D 7020 6C69 6272 6172 7920 |hexdump library |
00000040: 666F 7220 5275 7374 2E                  |for Rust.       |
```

## Features

 - Convenient: Hexd tries to be "at your fingerprints" by exposing [blanket conversion traits](https://docs.rs/hxd/latest/hxd/trait.AsHexd.html) and an intuitive [builder-based options API](https://docs.rs/hxd/latest/hxd/options/trait.HexdOptionsBuilder.html).
 - Dependency-free: this crate aims to be as lightweight as possible and does not introduce any unwanted dependencies into your project.
 - Performant: this crate does not allocate more for larger hexdumps.
 - Flexible: hexdumps can be printed or collected to arbitrary types.
 - Easily extensible: this crate provides public traits for [reading from custom sources](https://docs.rs/hxd/latest/hxd/reader/trait.ReadBytes.html) and [writing to custom sinks](https://docs.rs/hxd/latest/hxd/writer/trait.WriteHexdump.html).

## Examples

Any slice of bytes [can be dumped](https://docs.rs/hxd/latest/hxd/trait.AsHexd.html) with a single line:
```rust
use hxd::AsHexd;
 
let v = (0..0x40).collect::<Vec<u8>>();
let msg = b"Hello, world! Hopefully you're seeing this in hexd...";

v.hexd().dump_to::<String>();
msg.hexd().dump();
// 00000000: 4865 6C6C 6F2C 2077 6F72 6C64 2120 486F |Hello, world! Ho|
// 00000010: 7065 6675 6C6C 7920 796F 7527 7265 2073 |pefully you're s|
// 00000020: 6565 696E 6720 7468 6973 2069 6E20 6865 |eeing this in he|
// 00000030: 7864 2E2E 2E                            |xd...           |
```

Any iterator that yields bytes can be [consumed and dumped](https://docs.rs/hxd/latest/hxd/trait.IntoHexd.html) as well:
```rust
use hxd::IntoHexd;

let msg = b"Hello, world! Hopefully you're seeing this in hexd...";
let iter = msg.into_iter().map(|u| *u + 1);

iter.hexd().dump();
// 00000000: 4966 6D6D 702D 2178 7073 6D65 2221 4970 |Ifmmp-!xpsme"!Ip|
// 00000010: 7166 6776 6D6D 7A21 7A70 7628 7366 2174 |qfgvmmz!zpv(sf!t|
// 00000020: 6666 6A6F 6821 7569 6A74 216A 6F21 6966 |ffjoh!uijt!jo!if|
// 00000030: 7965 2F2F 2F                            |ye///           |
```

[Options](https://docs.rs/hxd/latest/hxd/options/struct.HexdOptions.html) are configurable 
via a [fluent interface](https://docs.rs/hxd/latest/hxd/options/trait.HexdOptionsBuilder.html):

```rust
use hxd::{AsHexd, options::HexdOptionsBuilder, options::{GroupSize, Spacing}};

let v = (0..0x80).collect::<Vec<u8>>();

v.hexd()
    .grouped((GroupSize::Int, Spacing::None), (4, Spacing::Normal))
    .uppercase(false)
    .range(0x45..0x7b)
    .relative_offset(0xff0000)
    .dump();
// 00ff0040:            454647 48494a4b 4c4d4e4f |     EFGHIJKLMNO|
// 00ff0050: 50515253 54555657 58595a5b 5c5d5e5f |PQRSTUVWXYZ[\]^_|
// 00ff0060: 60616263 64656667 68696a6b 6c6d6e6f |`abcdefghijklmno|
// 00ff0070: 70717273 74757677 78797a            |pqrstuvwxyz     |
```

Hexdumps can be [written](https://docs.rs/hxd/latest/hxd/writer/trait.WriteHexdump.html) 
to a variety of targets out of the box:

```rust,no_run
use hxd::{AsHexd, options::HexdOptionsBuilder};
use std::{fs::{OpenOptions, File}, net::TcpStream};

let f = OpenOptions::new()
    .write(true)
    .open("hexdump.txt")
    .unwrap();

let tcp_stream = TcpStream::connect("127.0.0.1:9000").unwrap();

let v = vec![0u8; 16];

v.hexd().dump();
v.hexd().dump_err();
v.hexd().dump_to::<String>();
v.hexd().dump_to::<Vec<u8>>();
v.hexd().dump_io(f).unwrap();
v.hexd().dump_io(tcp_stream).unwrap();
```

Hexd can handle more than just bytes. All primitive integer types [can be dumped](https://docs.rs/hxd/latest/hxd/trait.AsHexdGrouped.html) with sensible display defaults:

```rust
use hxd::{AsHexdGrouped, options::Endianness};

vec![0x6120u16; 8].as_hexd(Endianness::LittleEndian).dump();
// 00000000: 2061 2061 2061 2061 2061 2061 2061 2061 | a a a a a a a a|

vec![0x7fa06120i32; 4].as_hexd_be().dump();
// 00000000: 7FA06120 7FA06120 7FA06120 7FA06120 |..a ..a ..a ..a |

vec![0xff3007fa06120u64; 2].as_hexd_le().dump();
// 00000000: 2061A07F00F30F00 2061A07F00F30F00 | a...... a......|

vec![0x7fa06120u128; 1].as_hexd_be().dump();
// 00000000: 00 00 00 00 00 00 00 00 00 00 00 00 7F A0 61 20 |..............a |
```

## License

This project is licensed under the [MIT license](https://github.com/benjdod/hexd/blob/master/LICENSE.txt).