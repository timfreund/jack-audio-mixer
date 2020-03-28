extern crate jack;
use std::io;
// use std::time::SystemTime;

struct Channel {
    name: String,
    level: f32,
    mute: bool,
    pan: f32,
    input: jack::Port<jack::AudioIn>,
}

struct Mixer {
    inputs: Vec<Channel>,
    // outputs: Vec<jack::Port<jack::AudioOut>>,
    output: jack::Port<jack::AudioOut>,
}

fn main() {
    println!("wtf");
    let(jack_client, _status) = jack::Client::new("jam", jack::ClientOptions::NO_START_SERVER).unwrap();

    let mut mixer = Mixer {
        inputs: Vec::new(),
        output: jack_client.register_port("out", jack::AudioOut::default()).unwrap(),
    };

    for i in 0..8 {
        mixer.inputs.push(Channel {
            name: i.to_string(),
            level: 0.1,
            mute: false,
            pan: 0.0,
            input: jack_client.register_port(&format!("in_{}", i), jack::AudioIn::default()).unwrap()
        });
    }    

    // for i in 0..2 {
    //     mixer.outputs.push(jack_client.register_port(&format!("out_{}", i), jack::AudioOut::default()).unwrap());
    // }    

    let jack_callback = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // println!("ugh");
            println!("{}", ps.last_frame_time());
            let output = mixer.output.as_mut_slice(ps);
            // let output_buffer = mixer.output.buffer(ps.n_frames());
            // zero out the slice before adding new stuff... seems that it's
            // the same memory reused between frames
            for ov in output.iter_mut() {
                *ov = 0.0;
            }
            for c in &mixer.inputs {
                if !c.mute {
                    let islice = c.input.as_slice(ps);
                    // println!("{} {} {}", c.name, c.level, c.pan);
                    // println!("{} {}", output.len(), islice.len());

                    let output_iter = output.iter_mut();
                    for (ov, iv) in output_iter.zip(islice){
                        // *ov = *ov + (c.level * iv);
                        *ov = *ov + (c.level * iv);
                    }
                    
                    // for (ov, iv) in output.iter().zip(islice) {
                    //     *ov = ov + (c.level * iv)
                    // }
                        
                    // let islice = c.input.as_slice(ps);
                    // for (iv, ov) in islice.iter().zip(output) {
                    //     *ov = *ov + (c.level * iv)
                    // }
                }
            }
            jack::Control::Continue
        },
    );

    // let jack_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
    //     let output = mixer.output.as_mut_slice(ps);
    //     for c in &mixer.inputs {
    //         if !c.mute {
    //             let islice = c.input.as_slice(ps);
    //             let output_iter = output.iter_mut();
    //             for (ov, iv) in output_iter.zip(islice){
    //                 // *ov = *ov + (c.level * iv);
    //                 *ov = *ov + iv;
    //             }                    
    //         }
    //     }
    //     jack::Control::Continue
    // };
    
    // let active_client = jack_client.activate_async(Notifications, jack_callback).unwrap();
    let active_client = jack_client.activate_async((), jack_callback).unwrap();

    println!("Press any key to quit");
    let mut quit = String::new();
    io::stdin().read_line(&mut quit).ok();

    active_client.deactivate().unwrap();
}

// struct Notifications;

// impl jack::NotificationHandler for Notifications {
//     fn thread_init(&self, _: &jack::Client) {
//         println!("JACK: thread init");
//     }

//     fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
//         println!(
//             "JACK: shutdown with status {:?} because \"{}\"",
//             status, reason
//         );
//     }

//     fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
//         println!(
//             "JACK: freewheel mode is {}",
//             if is_enabled { "on" } else { "off" }
//         );
//     }

//     fn buffer_size(&mut self, _: &jack::Client, sz: jack::Frames) -> jack::Control {
//         println!("JACK: buffer size changed to {}", sz);
//         jack::Control::Continue
//     }

//     fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
//         println!("JACK: sample rate changed to {}", srate);
//         jack::Control::Continue
//     }

//     fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
//         println!(
//             "JACK: {} client with name \"{}\"",
//             if is_reg { "registered" } else { "unregistered" },
//             name
//         );
//     }

//     fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
//         println!(
//             "JACK: {} port with id {}",
//             if is_reg { "registered" } else { "unregistered" },
//             port_id
//         );
//     }

//     fn port_rename(
//         &mut self,
//         _: &jack::Client,
//         port_id: jack::PortId,
//         old_name: &str,
//         new_name: &str,
//     ) -> jack::Control {
//         println!(
//             "JACK: port with id {} renamed from {} to {}",
//             port_id, old_name, new_name
//         );
//         jack::Control::Continue
//     }

//     fn ports_connected(
//         &mut self,
//         _: &jack::Client,
//         port_id_a: jack::PortId,
//         port_id_b: jack::PortId,
//         are_connected: bool,
//     ) {
//         println!(
//             "JACK: ports with id {} and {} are {}",
//             port_id_a,
//             port_id_b,
//             if are_connected {
//                 "connected"
//             } else {
//                 "disconnected"
//             }
//         );
//     }

//     fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
//         println!("JACK: graph reordered");
//         jack::Control::Continue
//     }

//     fn xrun(&mut self, _: &jack::Client) -> jack::Control {
//         println!("JACK: xrun occurred");
//         jack::Control::Continue
//     }

//     fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
//         println!(
//             "JACK: {} latency has changed",
//             match mode {
//                 jack::LatencyType::Capture => "capture",
//                 jack::LatencyType::Playback => "playback",
//             }
//         );
//     }
// }
