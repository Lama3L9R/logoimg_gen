use std::env::args;
use std::fs::{ File, OpenOptions };
use std::io::{ Read, Seek, Write };
use std::path::Path;

/*
 These offsets are the end of the part
*/
const HEADER_OFFSET: i32 = 20480; // + 1 Should be the first byte of Mi logo
const MI_LOGO_OFFSET: i32 = 7602176; // + 1 Should be the first byte of Fastboot logo
const FASTBOOT_LOGO_OFFSET: i32 = 15183872; // + 1 Should be the first byte of Unlocked logo
const UNLOCKED_LOGO_OFFSET: i32 = 22765568; // + 1 Should be the first byte of 'the system has been destroyed' logo
const MAX_SIZE: i32 = 7581696;

fn main() {
    println!("This is the initial version of the tool. USE IT AS YOUR OWN RISK! MAKE SURE TO CHECK THE OUTPUT FILE BEFORE USING IT!");

    let mut args = args();
    if args.len() < 6 {
        println!("Usage: {} <target> <mi> <fastboot> <unlocked> <system destroyed>", args.nth(0).unwrap());
        return;
    }
    let target = args.nth(1).expect("Please specify a target image location");
    let mi = args.next().expect("Please specify a mi logo");
    let fastboot = args.next().expect("Please specify a fastboot logo");
    let unlocked = args.next().expect("Please specify an unlocked logo");
    let system_destroyed = args.next().expect("Please specify a 'the system has been destroyed' logo");

    println!("Creating target image file: {}...", target);

    let mut target_file = OpenOptions::new().create(true)
        .write(true)
        .append(true)
        .open(&target).expect("Failed to open(create) target file");
    target_file.seek(std::io::SeekFrom::Start(HEADER_OFFSET as u64)).expect("Failed to seek to start of target file");

    write_image(&mut target_file, &mi, "Mi");
    write_image(&mut target_file, &fastboot, "Fastboot");
    write_image(&mut target_file, &unlocked, "Unlocked");
    write_image(&mut target_file, &system_destroyed, "System Destroyed");

    drop(target_file);

    println!("Done! 4 / 4 images was written into {}.", target);
    println!("Please check the file size and make sure it's correct. Expected: 30347264");
    println!("If it's correct, you can flash it to your device by using the following command.");
    println!("\n        fastboot flash logo {}\n", target);
}

fn write_image(target_file: &mut File, image: &str, logo_name: &str) -> Option<()> {
    return if let picture = Path::new(image) {
        println!("  Processing Logo: {}...", logo_name);
        let mut picture_file = File::open(picture).expect("Failed to open Mi logo file");
        picture_file.seek(std::io::SeekFrom::Start(0)).expect("Failed to seek to start of Mi logo file");
        let mut buffer = [0u8; 1024];
        let mut total = 0;
        println!("    Writing file...");
        loop {
            let read = picture_file.read(&mut buffer).expect("Failed to read Mi logo file");
            if read == 0 {
                break;
            }
            total += read;
            target_file.write(&buffer).expect("Failed to write Mi logo to target file");
        }

        if total as u64 > MAX_SIZE as u64 {
            println!("** ERROR: logo {} is too large! Provided: {} Bytes, Expected: {} Bytes **", total, MAX_SIZE, logo_name);
            return None;
        } else if total as u64 != MAX_SIZE as u64 {
            let padding = MAX_SIZE - total as i32;
            target_file.write(&vec![0u8; padding as usize]).expect("Failed to write padding to target file");
            println!("    {} Bytes of data pad to end of logo.", padding);
        } else {
            println!("    0 Bytes of data pad to end of logo.");
        }
        println!("  Logo {} written successfully!", logo_name);
        Some(())
    } else {
        println!("** ERROR: Logo file is missing: {} **", logo_name);
        None
    }
}