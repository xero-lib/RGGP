use std::{fs::{ copy, remove_file, File }, os::unix::fs::FileExt, path::PathBuf};

use clap::{Parser, ValueEnum};

#[derive(Parser, Clone, ValueEnum, Debug)]
enum Mode {
    GameBoy, GB,
    GameGear, GG,
    MasterSystem, SMS,
    Genesis, SG,
    MegaDrive, MD,
    Nintendo, NES,
    SuperNintendo, SNES,
}

const NES_CONVERSION: [char; 16] = ['A', 'P', 'Z', 'L', 'G', 'I', 'T', 'Y', 'E', 'O', 'X', 'U', 'K', 'S', 'V', 'N'];

#[derive(Parser, Debug)]
#[command(version, about = "Patch a ROM with Game Genie codes", long_about = None)]
struct Args {
    #[arg(value_name = "CODES")]
    codes: String,
    #[arg(value_name = "MODE", help = "ROM mode selection")]
    mode: Mode,
    #[arg(value_name = "INPUT", help = "Path to input ROM file")]
    rom_in: PathBuf,
    #[arg(value_name = "OUTPUT", help = "Desired output path for patched ROM")]
    rom_out: PathBuf,
}

fn parse_nes(code: &str, file: &File) -> (u16, u8) {
    if code.len() == 6 { //  unchecked code variation
        // convert code chars to predesignated usize values
        let data_hex: Vec<u8> = code.chars().map(|i| {
            match NES_CONVERSION.iter().position(|&c| c == i) {
                Some(x) => x as u8,
                None => {
                    eprintln!("Invalid code input");
                    std::process::exit(32);
                }
            }
        }).collect();

        // bit manupulation of u8s
        /*
            0000 1111 2222 3333 4444 5555
            -333 4555 1222 3444 0111 5000 
         */

        let mut res_data: [u8; 6] = [0, 0, 0, 0, 0, 0];
        
        res_data[0] = (data_hex[3] & 0b0111)                          as u8;
        res_data[1] = (data_hex[5] & 0b0111) + (data_hex[4] & 0b1000) as u8;
        res_data[2] = (data_hex[2] & 0b0111) + (data_hex[1] & 0b1000) as u8;
        res_data[3] = (data_hex[4] & 0b0111) + (data_hex[3] & 0b1000) as u8;
        res_data[4] = (data_hex[1] & 0b0111) + (data_hex[0] & 0b1000) as u8;
        res_data[5] = (data_hex[0] & 0b0111) + (data_hex[5] & 0b1000) as u8;
    
        let address: u16 = {
            ((res_data[0] as u16) << (4 * 3)) +
            ((res_data[1] as u16) << (4 * 2)) +
            ((res_data[2] as u16) << (4 * 1)) +
            ((res_data[3] as u16) << (4 * 0)) 
        };

        let value: u8 = (res_data[4] << 4) + res_data[5];

        return (address, value);
    } else if code.len() == 8 {
        let data_hex: Vec<u8> = code.chars().map(|i| {
            match NES_CONVERSION.iter().position(|&c| c == i) {
                Some(x) => x as u8,
                None => {
                    eprintln!("Invalid code input");
                    std::process::exit(32);
                }
            }
        }).collect();

        // bit manupulation of u8s
        /*
            0000 1111 2222 3333 4444 5555 6666 7777
            -333 4555 1222 3444 0111 7000 6777 5666
         */

        let mut res_data: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        
        res_data[0] = (data_hex[3] & 0b0111)                          as u8;
        res_data[1] = (data_hex[5] & 0b0111) + (data_hex[4] & 0b1000) as u8;
        res_data[2] = (data_hex[2] & 0b0111) + (data_hex[1] & 0b1000) as u8;
        res_data[3] = (data_hex[4] & 0b0111) + (data_hex[3] & 0b1000) as u8;
        res_data[4] = (data_hex[1] & 0b0111) + (data_hex[0] & 0b1000) as u8;
        res_data[5] = (data_hex[0] & 0b0111) + (data_hex[5] & 0b1000) as u8;
        res_data[6] = (data_hex[7] & 0b0111) + (data_hex[6] & 0b1000) as u8;
        res_data[7] = (data_hex[6] & 0b0111) + (data_hex[5] & 0b1000) as u8;
    
        let address: u16 = {
            ((res_data[0] as u16) << (4 * 3)) +
            ((res_data[1] as u16) << (4 * 2)) +
            ((res_data[2] as u16) << (4 * 1)) +
            ((res_data[3] as u16) << (4 * 0)) 
        };
        
        let mut check_nibble = [0_u8];
        file.read_at(&mut check_nibble[..], address as u64).expect("Unable to read ROM file");
        return if check_nibble[0] != res_data[7] { (address, check_nibble[0]) } else { (address, res_data[6]) };
    } else {
        panic!("Invalid code length")
    }

}

fn main() {
    let args = Args::parse();
    
    if !args.rom_in.is_file() {
        eprintln!("Unable to read {}", args.rom_in.display());
        return;
    }

    let codes: Vec<&str> = args.codes.split('+').collect();

    let _ = copy(&args.rom_in, &args.rom_out);

    let Ok(file) = File::options().write(true).read(true).open(&args.rom_out) else {
        eprintln!("Unable to open ROM file for reading");
        if args.rom_out.exists() { remove_file(args.rom_out).unwrap(); }
        return;
    };

    for code in codes {
        use Mode::*;
        let (offset, data) = match args.mode {
            #[allow(unused_parens)]
            ( GameBoy | GB ) | ( GameGear | GG ) | ( MasterSystem | SMS ) => { todo!("Unemplimented") },

            #[allow(unused_parens)]
            ( Genesis | SG ) | ( MegaDrive | MD ) => { todo!("Unemplimented") },
            
            Nintendo | NES => {
                let res = parse_nes(code, &file);
                (res.0 + 0x10, res.1) /* + 0x10 for header offset */
            },

            SuperNintendo | SNES => { todo!("Unemplimented") }
        };

        println!("Offset: {offset:x}\nData: {data:x}");

        
        file.write_at(&[data], offset as u64).expect("Unable to write code data to file");
    }
}
