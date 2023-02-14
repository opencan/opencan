use opencan_core::CANNetwork;

pub fn hello() {
    println!("Hello, world!");
}

// lifetime to save one reference to can netowrk

pub struct Analyze<'n> {
    net: &'n CANNetwork,
}

impl<'n> Analyze<'n> {
    pub fn new(net: &'n CANNetwork) -> Self {
        Self { net: net }
    }
    pub fn printnodes(&self) {
        let mut frames: u32 = 0;
        for msg in self.net.iter_messages() {
            let frame = 1000 / msg.cycletime.unwrap();
            frames += frame;
            println!("{} {:?} {}", msg.name, msg.cycletime, frame);
        }
        println!("Frames per second: {}", frames)
    }
}

fn eleven_bit_id(tbit:u8){
    Cm = (((34 + 8*8)/5) + 47 + 8*8)*tbit;
}

fn twenty_nine_bit_id(tbit:u8){
    Cm = (((52 + 8*8)/5) + 65 + 8*8)*tbit;
}