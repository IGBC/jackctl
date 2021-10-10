use alsa::pcm::{HwParams, PCM};
use alsa::Direction;
use regex::Regex;
use std::env;
use std::ffi::CString;
use std::fs;
use std::process::abort;

const EXTENDED_SAMPLE_RATES: [u32; 19] = [
    8000, 11025, 16000, 22050, 32000, 37800, 44056, 44100, 47250, 48000, 50000, 50400, 64000,
    88200, 96000, 176400, 192000, 352800, 384000,
];

fn check_playback(id: &str) {
    match PCM::new(id, Direction::Playback, false) {
        Ok(pcm) => {
            let hwp = HwParams::any(&pcm).unwrap();
            let start = hwp.get_channels_min().unwrap();
            let end = hwp.get_channels_max().unwrap() + 1;

            println!("    Playback channels: {}, {}", start, end - 1);
            //,         hwp.get_channels().unwrap());
            hwp.set_rate_resample(false).unwrap();
            for rate in EXTENDED_SAMPLE_RATES.iter() {
                let mut chans: Vec<u32> = Vec::new();

                for n in start..end {
                    if hwp.test_channels(n).is_ok() {
                        match hwp.test_rate(*rate) {
                            Ok(()) => {
                                chans.push(n);
                            }
                            Err(_) => (),
                        };
                    };
                }

                if !chans.is_empty() {
                    println!("        {}: Ok {:?}", rate, chans);
                }
            }
        }
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
            let end = hwp.get_channels_max().unwrap() + 1;

            println!("    Capture channels: {}, {}", start, end - 1);
            //,         hwp.get_channels().unwrap());
            hwp.set_rate_resample(false).unwrap();
            for rate in EXTENDED_SAMPLE_RATES.iter() {
                let mut chans: Vec<u32> = Vec::new();

                for n in start..end {
                    if hwp.test_channels(n).is_ok() {
                        match hwp.test_rate(*rate) {
                            Ok(()) => {
                                chans.push(n);
                            }
                            Err(_) => (),
                        };
                    };
                }

                if !chans.is_empty() {
                    println!("        {}: Ok {:?}", rate, chans);
                }
            }
        }
        Err(e) => {
            println!("   Capture - cannot open card: {}", e);
        }
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
                check_playback(&id);
                check_capture(&id);
            }
        }
        "sys" => {
            let sys = fs::read_dir("/sys/class/sound");
            match sys {
                Ok(sys) => {
                    for entry in sys {
                        let entry = entry.unwrap();
                        let fname = entry.file_name();

                        let fname_str = fname.to_str().unwrap();

                        let re =
                            Regex::new(r"pcmC(?P<card>\d+)D(?P<dev>\d+)(?P<mode>p|c)").unwrap();
                        let mat_ch = re.captures(fname_str);

                        match mat_ch {
                            Some(caps) => {
                                let path = entry.path();
                                let path_str = path.to_str().unwrap();

                                let card = format!("hw:{},{}", &caps["card"], &caps["dev"]);

                                let alsa_card = alsa::card::Card::from_str(
                                    &CString::new(&caps["card"]).unwrap(),
                                );

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
                    }
                }
                Err(e) => println!("cannot open /sys/class/sound - {}", e),
            }
        }
        _ => {
            print_usage();
        }
    }
}
