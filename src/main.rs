extern crate jack;
use std::io;

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
            let output = mixer.output.as_mut_slice(ps);

            // zero out the slice before adding new stuff... seems that it's
            // the same memory reused between frames
            for ov in output.iter_mut() {
                *ov = 0.0;
            }
            for c in &mixer.inputs {
                if !c.mute {
                    let islice = c.input.as_slice(ps);
                    // I wrestled with this a lot and it came down to
                    // iter() != iter_mut().  Once I got a mutable
                    // iterator everything snapped into place
                    let output_iter = output.iter_mut();
                    for (ov, iv) in output_iter.zip(islice){
                        // *ov = *ov + (c.level * iv);
                        *ov = *ov + (c.level * iv);
                    }
                }
            }
            jack::Control::Continue
        },
    );
    
    let active_client = jack_client.activate_async((), jack_callback).unwrap();

    println!("Press any key to quit");
    let mut quit = String::new();
    io::stdin().read_line(&mut quit).ok();

    active_client.deactivate().unwrap();
}
