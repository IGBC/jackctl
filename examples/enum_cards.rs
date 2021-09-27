use alsa::pcm::{HwParams, PCM};
use alsa::{Direction, Ctl, HCtl};
use alsa::device_name;
use std::env;
use std::process::abort;
use std::fs;

use regex::Regex;

use std::ffi::CString;

const EXTENDED_SAMPLE_RATES: [u32; 19] = [
    8000, 11025, 16000, 22050, 32000, 37800, 44056, 44100, 47250, 48000, 50000, 50400, 64000,
    88200, 96000, 176400, 192000, 352800, 384000,
];

fn check_playback(id: &str) {
    match PCM::new(id, Direction::Playback, false) {
        Ok(pcm) => {
            let hwp = HwParams::any(&pcm).unwrap();
            let start = hwp.get_channels_min().unwrap();
            let end = hwp.get_channels_max().unwrap()+1;

            println!("    Playback channels: min = {}, max = {}", start, end-1);
            //,         hwp.get_channels().unwrap());
            hwp.set_rate_resample(false).unwrap();
            for rate in 1..4000000 {
                let mut chans: Vec<u32> = Vec::new();
            
                for n in start..end {
                    if hwp.test_channels(n).is_ok() {
                        match hwp.test_rate(rate) {
                            Ok(()) => {
                                chans.push(n);
                            },
                            Err(_) => (),
                        };
                    };
                    
                }

                if !chans.is_empty() {
                    println!("        {}: Ok {:?}", rate, chans);
                }
            }
        },
        Err(e) => {
            println!("   Playback - cannot open card: {}", e);
        }
    }
}
            
fn check_capture(id: &str) {    
    match PCM::new(id, Direction::Capture, false) {
        Ok(pcm) => {
            let hwp = HwParams::any(&pcm).unwrap();
            let start = hwp.get_channels_min().unwrap();
            let end = hwp.get_channels_max().unwrap()+1;

            println!("    Capture channels: min = {}, max = {}", start, end-1);
            //,         hwp.get_channels().unwrap());
            hwp.set_rate_resample(false).unwrap();
            for rate in 1..4000000 {
                let mut chans: Vec<u32> = Vec::new();
            
                for n in start..end {
                    if hwp.test_channels(n).is_ok() {
                        match hwp.test_rate(rate) {
                            Ok(()) => {
                                chans.push(n);
                            },
                            Err(_) => (),
                        };
                    };
                }

                if !chans.is_empty() {
                    println!("        {}: Ok {:?}", rate, chans);
                }
            }
        },
        Err(e) => {
            println!("   Capture - cannot open card: {}", e);
        }
    }
}


fn check_hctls(id: &str) {
    match HCtl::open(&CString::new(id).unwrap(), false) {
        Ok(h) => {
            h.load().unwrap();
            println!("    HCtls:");
            for b in h.elem_iter() {
                let id = b.get_id().unwrap();
                let info = b.info().unwrap();
                let int = id.get_interface();
                let name = id.get_name().unwrap();
                println!("        {} ({},{} {} {}) {:?} - {} x {:?}", 
                    name,
                    id.get_device(),
                    id.get_subdevice(),
                    id.get_numid(),
                    id.get_index(),
                    int,
                    info.get_count(),
                    info.get_type(),
                )
            }
        },
        Err(e) => {
            println!("   HCtl - cannot open card: {}", e);
        }
    }
    
}

fn check_ctls(id: &str) {
    println!("    Ctl: ");
    let a_info = Ctl::new(&id, false).unwrap().card_info().unwrap();
    println!("                id: {}", a_info.get_id().unwrap());
    println!("              name: {}", a_info.get_name().unwrap());
    println!("          longname: {}", a_info.get_longname().unwrap());
    println!("         mixername: {}", a_info.get_mixername().unwrap());
    println!("            driver: {}", a_info.get_driver().unwrap());
    println!("        components: {}", a_info.get_components().unwrap());
}

fn check_names(id: &str) {
    let alsa_card = Ctl::new(&id, false).unwrap().card_info().unwrap().get_card();
    for t in &["pcm", "ctl", "rawmidi", "timer", "seq", "hwdep"] {
        println!("    {} devices:", t);
        let i = device_name::HintIter::new(Some(&alsa_card), &*CString::new(*t).unwrap()).unwrap();
        for a in i { println!("        [{}]: \"{}\" - Direction {}", a.name.unwrap(), a.desc.unwrap().replace('\n', " "), a.direction.map(|x| format!("{:?}", x)).unwrap_or("None".to_owned())) }
    }
}

fn print_usage() {
    eprintln!("usage: enum_cards [alsa|sys]");
    abort();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
    }

    match args[1].as_str() {
        "alsa" => {
            for a in ::alsa::card::Iter::new().map(|x| x.unwrap()) {
                println!("hw:{} - {}", a.get_index(), a.get_name().unwrap());
                let id = format!("hw:{}", a.get_index());
                check_ctls(&id);
                check_hctls(&id);
                check_names(&id);
                check_playback(&id);
                check_capture(&id);
            }
        },
        "sys" => {
            let sys = fs::read_dir("/sys/class/sound");
            match sys {
                Ok(sys) => for entry in sys {
                    let entry = entry.unwrap();
                    let fname = entry.file_name();

                    let fname_str = fname.to_str().unwrap();

                    let re = Regex::new(r"pcmC(?P<card>\d+)D(?P<dev>\d+)(?P<mode>p|c)").unwrap();
                    let mat_ch = re.captures(fname_str);

                    match mat_ch {
                        Some(caps) => {
                            let path = entry.path();
                            let path_str = path.to_str().unwrap();

                            let card = format!("hw:{},{}", &caps["card"], &caps["dev"]);

                            let alsa_card = alsa::card::Card::from_str(&CString::new(&caps["card"]).unwrap());

                            match alsa_card {
                                Ok(c) => println!("{} - {}", card, c.get_name().unwrap()),
                                Err(e) => println!("Cannot open {} : {}", card, e),
                            }
                            
                            if &caps["mode"] == "p" {
                                check_playback(&card);
                            } else {
                                check_capture(&card);
                            }
                        }
                        None => {}
                    }
                },
                Err(e)  => println!("cannot open /sys/class/sound - {}", e)
            }
        },
        _ => {
            print_usage();
        }
    }
}
            
            

                            