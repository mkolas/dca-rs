#[macro_use(value_t)]
extern crate clap;

extern crate opus;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate byteorder;


mod structs;

use clap::{Arg, App};
use structs::*;
use std::env;
use opus::{Encoder, Channels, CodingMode};
use std::process::{Command, Stdio};
use std::path::Path;
use std::collections::BTreeMap;
use std::io::{self, Write, BufWriter, BufReader};
use std::thread;
use std::sync::mpsc;
use std::default::Default;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

const FORMAT_FILE_VERSION: i8 = 1; // Magic bytes version
const FORMAT_JSON_VERSION: i8 = 1; // JSON spec version

const VERSION: &'static str = "0.1.0";
const GITHUB_URL: &'static str = "https://github.com/nstafie/dca-rs";

fn parse_args<'a>() -> clap::ArgMatches<'a> {

    fn is_within_bitrate_range(v: String) -> Result<(), String> {
        let value = v.parse::<u32>().unwrap();
        if (value < 1) || (value > 512) {
            Err(String::from("Bitrate provided is not valid, it must be between 8 and 128 kb/s"))
        } else {
            Ok(())
        }
    }

    let mut args: Vec<String> = env::args().collect();

    let app = App::new("dca-rs")
        .version(VERSION)
        .arg(Arg::with_name("application")
            .short("a")
            .long("aa")
            .help("audio application can be voip, audio, or lowdelay")
            .takes_value(true)
            .value_name("application")
            .possible_values(&["voip", "audio", "lowdelay"])
            .default_value("audio"))
        .arg(Arg::with_name("bitrate")
            .short("b")
            .long("ab")
            .help("audio encoding bitrate in kb/s can be 8 - 128")
            .takes_value(true)
            .value_name("bitrate")
            .default_value("64")
            .validator(is_within_bitrate_range))
        .arg(Arg::with_name("channels")
            .short("c")
            .long("ac")
            .help("audio channels, 1 for mono 2 for stereo")
            .takes_value(true)
            .value_name("channels")
            .default_value("2")
            .possible_values(&["1", "2"]))
        .arg(Arg::with_name("sample rate")
            .short("r")
            .long("ar")
            .help("audio sample rate")
            .takes_value(true)
            .value_name("sampling rate")
            .default_value("48000"))
        .arg(Arg::with_name("frame size")
            .short("s")
            .long("as")
            .help("audio frame size can be 960 (20ms), 1920 (40ms), or 2880 (60ms)")
            .takes_value(true)
            .value_name("frame size")
            .default_value("960")
            .possible_values(&["960", "1920", "2880"]))
        .arg(Arg::with_name("cover format")
            .short("f")
            .long("cf")
            .help("format the cover art will be encoded with")
            .takes_value(true)
            .value_name("format")
            .default_value("jpeg"))
        .arg(Arg::with_name("input")
            .short("i")
            .long("i")
            .help("input file")
            .takes_value(true)
            .value_name("file")
            .required(true)
            .default_value("pipe:0"))
        .arg(Arg::with_name("volume")
            .short("v")
            .long("vol")
            .help("change audio volume (256=normal)")
            .takes_value(true)
            .value_name("level")
            .default_value("256"))
        /*.arg(Arg::with_name("output")
          .short("o")
          .long("output")
          .help("specify an output file (leave blank for stdout output)")
          .takes_value(true)
          .value_name("file"))*/
        .arg(Arg::with_name("raw")
            .long("raw")
            .help("raw opus output (no metadata or magic bytes)"));

    if args.len() == 1 {
        app.print_help().unwrap();
        std::process::exit(0);
    }

    if args.len() == 2 {
        // assume single argument is a file
        args.insert(1, "--i".to_owned());
    }

    app.get_matches_from(args)
}

fn main() {
    let matches = parse_args();

    // === Validation ===
    let input = matches.value_of("input").unwrap();
    if input == "pipe:0" {
        // input is a stdin pipe, assume the pipe is valid for now
        // TODO: check if the pipe is valid and open
    } else {
        // input is a file, check if it exists
        let path = Path::new(&input);
        path.metadata().expect("File does not exist or I do not have permissions for it");
    }

    // === Calculate needed values ===
    let frame_size = value_t!(matches, "frame size", u32).unwrap();
    let channels = value_t!(matches, "channels", u32).unwrap();
    let max_bytes = (frame_size * channels) * 2; // max size of opus frame in bytes
    let opus_frame_size: usize = (frame_size * channels) as usize;
    // size of the frames we create from PCM input

    // === Create opus encoder instance ===
    let sample_rate = value_t!(matches, "sample rate", u32).unwrap();
    let opus_channels = match channels {
        1 => Channels::Mono,
        2 => Channels::Stereo,
        _ => panic!("Failed to match channels."),
    };
    let coding_mode = match matches.value_of("application").unwrap() {
        "voip" => CodingMode::Voip,
        "audio" => CodingMode::Audio,
        "lowdelay" => CodingMode::LowDelay,
        _ => panic!("Failed to match coding mode"),
    };
    let bitrate = value_t!(matches, "bitrate", u32).unwrap();

    let raw_output = matches.is_present("raw");

    let mut metadata_json: String = Default::default();
    let mut metadata_json_size: i32 = Default::default();

    if !raw_output {

        let mut metadata: Metadata;

        metadata = Metadata {
            dca: DCAMetadata {
                version: FORMAT_JSON_VERSION,
                tool: DCAToolMetadata {
                    name: String::from("dca-rs"),
                    version: String::from(VERSION),
                    url: String::from(GITHUB_URL),
                    author: String::from("nstafie"),
                },
            },
            song_info: SongMetadata { ..Default::default() },
            origin: OriginMetadata { ..Default::default() },
            opus: OpusMetadata {
                bitrate: bitrate * 1000,
                sample_rate: value_t!(matches, "sample rate", u32).unwrap(),
                application: matches.value_of("application").unwrap().to_owned(),
                frame_size: frame_size,
                channels: channels,
            },
            extra: BTreeMap::new(),
        };

        // === ffprobe ===
        if input != "pipe:0" {
            let ffprobe = Command::new("ffprobe")
                .arg("-v")
                .arg("quiet")
                .arg("-print_format")
                .arg("json")
                .arg("-show_format")
                .arg(&input)
                .output()
                .unwrap_or_else(|e| panic!("Failed to run ffprobe: {}", e));
            let ffprobe_output = std::str::from_utf8(&ffprobe.stdout).unwrap();
            let ffprode_json: FFProbeData = serde_json::from_str(&ffprobe_output).unwrap();
            let ffprobe_tags = ffprode_json.format.tags.unwrap_or(FFProbeTags { ..Default::default() });

            metadata.song_info = SongMetadata {
                album: ffprobe_tags.album.unwrap_or(String::new()),
                comments: ffprobe_tags.comment.unwrap_or(String::new()),
                genre: ffprobe_tags.genre.unwrap_or(String::new()),
                artist: ffprobe_tags.artist.unwrap_or(String::new()),
                title: ffprobe_tags.title.unwrap_or(String::new()),
                cover: String::new(), // TODO: add cover image support
            };

            metadata.origin = OriginMetadata {
                source: String::from("file"),
                channels: channels,
                url: String::new(),
                encoding: ffprode_json.format.format_long_name.unwrap_or(String::new()),
                bitrate: ffprode_json.format.bitrate.unwrap_or(String::from("0")).parse::<u32>().unwrap_or(0),
            }
        } else {
            metadata.origin = OriginMetadata {
                source: "pipe".to_owned(),
                channels: channels,
                encoding: "pcm16/s16le".to_owned(),
                ..Default::default()
            };
        }

        metadata_json = serde_json::to_string(&metadata).unwrap();
        metadata_json_size = metadata_json.len() as i32;
    }

    // channels
    let (encoder_tx, encoder_rx) = mpsc::channel();
    let (output_tx, output_rx) = mpsc::channel();

    // opus encoder thread
    let settings = EncoderSettings {
        sample_rate: sample_rate,
        opus_channels: opus_channels,
        coding_mode: coding_mode,
        bitrate: bitrate as i32,
        max_bytes: max_bytes as usize,
    };
    let encoder = thread::spawn(move || {
        encoder_thread(encoder_rx, output_tx, settings);
    });

    let settings = OutputSettings {
        raw_output: raw_output,
        format_file_version: FORMAT_FILE_VERSION,
        metadata_json_size: metadata_json_size,
        metadata_json: metadata_json,
    };
    // stdout output thread
    let output = thread::spawn(move || {
        output_thread(output_rx, settings);
    });

    if input != "pipe:0" {
        // === ffmpeg ===
        let volume = value_t!(matches, "volume", String).unwrap();
        let sample_rate = value_t!(matches, "sample rate", String).unwrap();
        let channels_str = value_t!(matches, "channels", String).unwrap();
        let ffmpeg = Command::new("ffmpeg")
            .arg("-i")
            .arg(&input)
            .arg("-vol")
            .arg(&volume)
            .arg("-f")
            .arg("s16le")
            .arg("-ar")
            .arg(&sample_rate)
            .arg("-ac")
            .arg(&channels_str)
            .arg("pipe:1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|e| panic!("Failed to run ffmpeg: {}", e));

        let mut input_pipe = BufReader::new(ffmpeg.stdout.unwrap());

        loop {

            let mut i16_buffer: Vec<i16> = Vec::new();
            for _ in 0..opus_frame_size {
                match input_pipe.read_i16::<LittleEndian>() {
                    Ok(val) => i16_buffer.push(val),
                    Err(_) => {
                        break;
                    }
                }
            }
            if i16_buffer.len() == 0 {
                break;
            }

            if i16_buffer.len() < opus_frame_size as usize {
                loop {
                    match i16_buffer.len() % opus_frame_size {
                        0 => break,
                        _ => i16_buffer.push(0),

                    }
                }
            }
            encoder_tx.send(i16_buffer).unwrap();
        }

    } else {
        let mut input_pipe = BufReader::new(io::stdin());
        loop {

            let mut i16_buffer: Vec<i16> = Vec::new();
            for _ in 0..opus_frame_size {
                match input_pipe.read_i16::<LittleEndian>() {
                    Ok(val) => i16_buffer.push(val),
                    Err(_) => {
                        break;
                    }
                }
            }
            if i16_buffer.len() == 0 {
                break;
            }

            if i16_buffer.len() < opus_frame_size as usize {
                loop {
                    match i16_buffer.len() % opus_frame_size {
                        0 => break,
                        _ => i16_buffer.push(0),

                    }
                }
            }
            encoder_tx.send(i16_buffer).unwrap();
        }

    }
    drop(encoder_tx);
    encoder.join().unwrap();
    output.join().unwrap();
}

struct EncoderSettings {
    sample_rate: u32,
    opus_channels: Channels,
    coding_mode: CodingMode,
    bitrate: i32,
    max_bytes: usize,
}

fn encoder_thread(input: mpsc::Receiver<Vec<i16>>,
                  output: mpsc::Sender<Vec<u8>>,
                  settings: EncoderSettings) {
    let mut opus_encoder = match Encoder::new(settings.sample_rate,
                                              settings.opus_channels,
                                              settings.coding_mode) {
        Ok(e) => e,
        Err(err) => panic!(err),
    };
    opus_encoder.set_bitrate(settings.bitrate * 1000).unwrap();
    loop {
        let frame = match input.recv() {
            Ok(f) => f,
            Err(_) => {
                break; // no more messages can be sent on this channel
            }
        };
        let mut f = vec![0u8; settings.max_bytes];
        let len = opus_encoder.encode(&frame, &mut f).unwrap();
        f.truncate(len as usize);
        output.send(f).unwrap();
    }
}

struct OutputSettings {
    raw_output: bool,
    format_file_version: i8,
    metadata_json_size: i32,
    metadata_json: String,
}

fn output_thread(input: mpsc::Receiver<Vec<u8>>, settings: OutputSettings) {
    let mut stdout = BufWriter::new(io::stdout());
    if !settings.raw_output {
        stdout.write(format!("DCA{}", settings.format_file_version).as_bytes()) // magic bytes
            .unwrap();
        stdout.write_i32::<LittleEndian>(settings.metadata_json_size).unwrap(); // json size
        stdout.write(settings.metadata_json.as_bytes()).unwrap(); // json data
    }
    loop {
        let mut output: Vec<u8> = Vec::new();
        let input_data = match input.recv() {
            Ok(i) => i,
            Err(_) => {
                break; // no more messages can be sent on this channel
            }
        };
        let frame_length: i16 = input_data.len() as i16;
        output.write_i16::<LittleEndian>(frame_length).unwrap();
        output.extend_from_slice(&input_data);
        stdout.write(&output[..]).unwrap();
    }
}
