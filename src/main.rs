extern crate byteorder;

use ascii::AsciiString;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Error, ErrorKind};
use std::str;
use std::{
    fs::File,
    io::{self, Read},
};
use structopt::StructOpt; // 1.2.7

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

struct Star {
    x: i32,
    y: i32,
    z: i32,
    index: i32,
    _unused: i32,
    name: AsciiString,
    typestr: AsciiString,
}

impl Star {
    fn from_reader(mut rdr: impl Read) -> io::Result<Self> {
        let x = rdr.read_i32::<LittleEndian>()?;
        let y = rdr.read_i32::<LittleEndian>()?;
        let z = rdr.read_i32::<LittleEndian>()?;
        let index = rdr.read_i32::<LittleEndian>()?;
        let _unused = rdr.read_i32::<LittleEndian>()?;
        let mut _name = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut _typestr = vec![0, 0, 0, 0];
        for i in 0..20 {
            _name[i] = rdr.read_u8()?;
        }
        for i in 0..4 {
            _typestr[i] = rdr.read_u8()?;
        }
        let name_r = AsciiString::from_ascii(_name);
        let typestr_r = AsciiString::from_ascii(_typestr);
        
        let typestr: AsciiString;
        let name: AsciiString;
        match name_r {
            Ok(str) => {
                name = str;
            },
            Err(_) => {
                return Err(Error::new(ErrorKind::Other, "name not ascii!?"))
            }
        }
        match typestr_r {
            Ok(str) => {
                typestr = str;
            },
            Err(_) => {
                return Err(Error::new(ErrorKind::Other, format!("typestr not ascii!? (entity name: {})", name)))
            }
        }

        Ok(Star {
            x,
            y: -y,
            z,
            index,
            _unused,
            name,
            typestr,
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();
    let file = File::open(&args.path).unwrap();

    let mut i = 0;
    loop {
        let star = Star::from_reader(&file);
        i = i + 1;
        match star {
            Ok(star) => {
                println!(
                    "Star/Planet: [{}] {} {} {} {} {} {}",
                    i, star.x, star.y, star.z, star.name, star.index, star.typestr
                );
            }
            Err(error) => {
                // malformed entry OR EOF
                // exit if EOF, ignore otherwise
                if error.kind() == ErrorKind::UnexpectedEof {
                    return Ok(());
                }
                eprintln!("Malformed entry at [{}]; {}", i, error.to_string());
            }
        };
    }
}
