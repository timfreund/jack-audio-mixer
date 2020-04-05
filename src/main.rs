extern crate clap;
extern crate crossbeam_channel;
extern crate jack;
extern crate rosc;

use clap::{Arg, App};
use crossbeam_channel::unbounded;
use rosc::{OscPacket, OscType};
use std::net::UdpSocket;

struct Channel {
    name: String,
    level: f32,
    mute: bool,
    pan: f32,
    input: jack::Port<jack::AudioIn>,
}

struct Mixer {
    inputs: Vec<Channel>,
    outputs: Vec<jack::Port<jack::AudioOut>>,
}

fn main() {
    let args = App::new("jack-audio-mixer")
        .version("0.1")
        .author("Tim Freund <tim@freunds.net>")
        .about("Mix multiple audio inputs")
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Channel configuration file")
             .takes_value(true))
        .arg(Arg::with_name("inputs")
             .short("i")
             .long("inputs")
             .value_name("INTEGER")
             .help("Number of input channels")
             .default_value("8")
             .takes_value(true))
        .arg(Arg::with_name("outputs")
             .short("o")
             .long("outputs")
             .value_name("INTEGER")
             .help("Number of output channels")
             .default_value("2")
             .takes_value(true))
        .get_matches();

    
    match args.value_of("config"){
        None => println!("No configuration file provided"),
        Some(config_file_path) => {
            println!("Should load configuration file {}", config_file_path);
        }
    }

    let mut output_channel_count = 2;
    match args.value_of("outputs") {
        Some(occ) => {
            output_channel_count = occ.parse::<i32>().unwrap();
        },
        _ => {

        }
    }

    // TODO: does this need to be mutable or is there a
    // better pattern for this?
    let mut input_channel_count = 2;
    match args.value_of("inputs") {
        Some(icc) => {
            input_channel_count = icc.parse::<i32>().unwrap();
        },
        _ => {

        }
    }
    
    let(jack_client, _status) = jack::Client::new("jam", jack::ClientOptions::NO_START_SERVER).unwrap();

    let mut mixer = Mixer {
        inputs: Vec::new(),
        outputs: Vec::new(),
    };

    for i in 0..output_channel_count {
        mixer.inputs.push(Channel {
            name: i.to_string(),
            level: 0.1,
            mute: false,
            pan: 0.0,
            input: jack_client.register_port(&format!("in_{}", i), jack::AudioIn::default()).unwrap()
        });
    }    

    for i in 0..input_channel_count {
        mixer.outputs.push(jack_client.register_port(&format!("out_{}", i), jack::AudioOut::default()).unwrap());
    }

    let (tx, rx) = unbounded();

    let jack_callback = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            while let Ok(msg) = rx.try_recv() {
                println!("from OSC socket {}", msg);
                // let msg_iter = msg.to_string().rsplit(' ');
                let msg_str = String::from(msg);
                let mut msg_iter = msg_str.split(' ');
                let chan = msg_iter.next().unwrap().parse::<i32>().unwrap();
                let attr = msg_iter.next().unwrap();
                let value = msg_iter.next().unwrap().parse::<f32>().unwrap();
                println!("from OSC socket: set {}.{} to {}", chan, attr, value);
                if attr == "level" {
                    mixer.inputs[chan as usize].level = value;
                } else if attr == "mute" {
                    if value == 1.0 {
                        mixer.inputs[chan as usize].mute = true;
                    } else if value == 0.0 {
                        mixer.inputs[chan as usize].mute = false;
                    }
                }
            }

            // &mut mixer.output works, but I need to understand why
            for output in &mut mixer.outputs {
                let os = output.as_mut_slice(ps);

                // zero out the slice before adding new stuff... seems that it's
                // the same memory reused between frames
                for ov in os.iter_mut() {
                    *ov = 0.0;
                }
                for c in &mixer.inputs {
                    if !c.mute {
                        let islice = c.input.as_slice(ps);
                        // I wrestled with this a lot and it came down to
                        // iter() != iter_mut().  Once I got a mutable
                        // iterator everything snapped into place
                        let output_iter = os.iter_mut();
                        for (ov, iv) in output_iter.zip(islice){
                            // *ov = *ov + (c.level * iv);
                            *ov = *ov + (c.level * iv);
                        }
                    }
                }
            }
            jack::Control::Continue
        },
    );
    
    let active_client = jack_client.activate_async((), jack_callback).unwrap();

    let oscsock = UdpSocket::bind("0.0.0.0:10888").unwrap();
    let mut oscbuf = [0u8; rosc::decoder::MTU];

    loop {
        match oscsock.recv_from(&mut oscbuf){
            Ok((size, addr)) => {
                println!("{} {}", size, addr);
                let packet = rosc::decoder::decode(&oscbuf[..size]).unwrap();
                match packet {
                    OscPacket::Message(msg) => {
                        println!("\t{} {:?}", msg.addr, msg.args);
                        if msg.addr.contains("/input/"){
                            let target = msg.addr.replace("/input/", "");
                            let components: Vec<&str> = target.splitn(2, "/").collect();
                            let chan = components[0].parse::<i32>().unwrap();
                            let attr = components[1];
                            println!("\t{} {}", chan, attr);

                            if attr == "level" || attr == "mute" {
                                match msg.args[0] {
                                    OscType::Float(f) => {
                                        let chan_index = chan - 1;
                                        tx.send(format!("{} {} {}", chan_index, attr, f)).unwrap();
                                        // // do I need to do some message passing like in the sine example here?
                                        // let mut channel = &mixer.inputs[index as usize];
                                        // channel.level = f;
                                    }
                                    _ => {
                                        // we only know how to handle floats
                                    }
                                }
                            }
                        }
                    }
                    OscPacket::Bundle(bundle) => {
                        println!("\t{:?}", bundle);
                    }
                }
            }
            Err(e) => {
                println!("Error (socket): {}", e);
                break;
            }
        }
    }

    // println!("Press any key to quit");
    // let mut quit = String::new();
    // io::stdin().read_line(&mut quit).ok();

    active_client.deactivate().unwrap();
}
