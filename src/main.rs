#[macro_use]
extern crate lazy_static;
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
use regex::Regex;
use raylib::prelude::*;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

// NOTE: actually a struct for *objects*, i.e. stars (if typestr starts with S) or planets (if typestr starts with P)
struct Star {
    x: i32,
    y: i32,
    z: i32,
    index: i32,
    _unused: i32,
    name: AsciiString,
    typestr: AsciiString,
}

impl std::convert::From<&Star> for raylib::ffi::Vector3 {
    fn from(star: &Star) -> Self {
        return raylib::ffi::Vector3 {x: star.x  as f32 * 0.0000001, y: star.y as f32 * 0.0000001, z: star.z as f32 * 0.0000001};
    }
}

lazy_static! {
    static ref TYPESTR_REGEX: Regex = Regex::new(r"^(P([0-9][0-9])|(S((0[0-9])|(1[0-1]))))$").unwrap();
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
                name = AsciiString::from(str.trim());
            },
            Err(_) => {
                return Err(Error::new(ErrorKind::Other, "name not ascii!?"))
            }
        }
        match typestr_r {
            Ok(str) => {
                typestr = AsciiString::from(str.trim());
            },
            Err(_) => {
                return Err(Error::new(ErrorKind::Other, format!("typestr not ascii!? (entity name: {})", name)))
            }
        }

        if !TYPESTR_REGEX.is_match(typestr.as_str()) {
            return Err(Error::new(ErrorKind::Other, format!("invalid typestr (typestr: {})", typestr)))
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
    let mut stars: Vec<Star> = Vec::new();
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
                if star.typestr[0] == 'S' {
                    stars.push(star);
                }
            }
            Err(error) => {
                // malformed entry OR EOF
                // exit if EOF, ignore otherwise
                if error.kind() == ErrorKind::UnexpectedEof {
                    break;
                }
                eprintln!("Malformed entry at [{}]; {}", i, error.to_string());
            }
        };
    }

    let (mut rl, thread) = raylib::init()
    .size(1600, 900)
    .title("Starmap2 viewer")
    .build();

    let mut camera = Camera3D::perspective(
        Vector3::new(4.0, 4.0, 4.0),  // Position
        Vector3::new(0.0, 0.0, 0.0), // Target
        Vector3::new(0.0, 1.0, 0.0),  // Up vector
        45.0,                         // FOV
    );

    let up_vec = raylib::ffi::Vector3 { x: 0.0, y: 1.0, z: 0.0};

    let disp_str = format!("Displaying coordinates of {} named objects", stars.len());
 
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        
        d.clear_background(Color::WHITE);
        d.draw_text(disp_str.as_str(), 12, 32, 20, Color::BLACK);
        d.draw_fps(12, 12);

        {
            let mut mode_3d = d.begin_mode3D(camera);

            mode_3d.draw_grid(10, 1.0);
            for s in &stars {
                //mode_3d.draw_point3D(s, Color::BLACK); // note: displays ugly as a line... :(
                mode_3d.draw_circle_3D(s, 0.005, up_vec, 0.0, Color::BLACK)
            }
        }
 
    }

    return Ok(());
}
