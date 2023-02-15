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
    //https://electronics.stackexchange.com/questions/422998/how-to-calculate-bus-load-of-can-bus
    pub fn printnodes(&self) {
        let mut frames: u32 = 0;
        let tbit = 1; // Mbit/s = 1 bit/microsecond = 1 MHz
        //for 29 bit id, we need something in the .yml or some other settings file to specify longer msg ids
        //i thought about reading the msg id with some function from compose, but that would require the existence of a msg with the highest id
        // im forcing it to 11 here
        let id_len = 11; 
        let mut bits_per_frame=0;
        let cap = tbit * 1000000;
        let mut busload  = 0.0;

        let mut max_sent = 0;
        for msg in self.net.iter_messages() {
            let frame = 1000 / msg.cycletime.unwrap();
            frames += frame;
            //println!("{} {:?} {}", msg.name, msg.cycletime, frame);
        }
        if id_len == 11{
            //Cm = (((34 + 8*8)/5) + 47 + 8*8)*tbit;
            // Cm = 130.6 -> constant for typical can frame w 11 bit id and 64 bit msg
            //tindell says 130, im gna round to 131 tho ...
            bits_per_frame = 131;
        }
        println!("Frames sent per second: {}", frames);
        max_sent = frames * bits_per_frame;
        println!("Max bits sent per sec: {}", max_sent);

        busload = ((max_sent as f64)/(cap as f64))*100.0;

        println!("Busload at {}%", busload)

    }
}


//for 29 bit id, we need something in the .yml or some other settings file to specify longer msg ids
//i thought about reading the msg id with some function from compose, but that would require the existence of a msg with the highest id
// fn twenty_nine_bit_id(tbit:u8){
//     let Cm;
//     Cm = (((52 + 8*8)/5) + 65 + 8*8)*tbit;
// }