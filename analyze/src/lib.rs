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
        Self { net }
    }
    //https://electronics.stackexchange.com/questions/422998/how-to-calculate-bus-load-of-can-bus
    pub fn printnodes(&self) {
        let mut frames: u32 = 0;
        let tbit = 1; // Mbit/s = 1 bit/microsecond = 1 MHz
                      //for 29 bit id, we need something in the .yml or some other settings file to specify longer msg ids
                      //i thought about reading the msg id with some function from compose, but that would require the existence of a msg with the highest id
                      // im forcing it to 11 here
        let id_len = 11;
        let cap = tbit * 1000000;

        let mut bits_sent = 0;

        for msg in self.net.iter_messages() {
            if let Some(cycletime) = msg.cycletime {
                let frame = 1000 / cycletime;
                frames += frame;
                if id_len == 11 {
                    let frame_bytes = msg.length;
                    let bits_per_frame = ((34 + 8 * frame_bytes) / 5) + 47 + 8 * frame_bytes;
                    bits_sent += frame * bits_per_frame
                }
            }
        }

        println!("Frames sent per second: {frames}");
        println!("Max bits sent per sec: {bits_sent}");

        let busload = ((bits_sent as f64) / (cap as f64)) * 100.0;

        println!("Busload at {busload}%")
    }
}


// fn twenty_nine_bit_id(tbit:u8){
//     let Cm;
//     Cm = (((52 + 8*8)/5) + 65 + 8*8)*tbit;
// }
