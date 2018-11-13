use std::env;
use std::fs;

const GPS_REGISTER_PROPS: [(u16, u16); 24] = [
    (2, 6),
    (3, 7),
    (4, 8),
    (5, 9),
    (1, 9),
    (2, 10),
    (1, 8),
    (2, 9),
    (3, 10),
    (2, 3),
    (3, 4),
    (5, 6),
    (6, 7),
    (7, 8),
    (8, 9),
    (9, 10),
    (1, 4),
    (2, 5),
    (3, 6),
    (4, 7),
    (5, 8),
    (6, 9),
    (1, 3),
    (4, 6),
];
const CODE_LENGTH: u16 = 1024;

const TOP_PUSH_REG: u16 = 2;
const BOT_PUSH_REG_1: u16 = 1;
const BOT_PUSH_REG_2: u16 = 2;
const BOT_PUSH_REG_3: u16 = 5;
const BOT_PUSH_REG_4: u16 = 7;
const BOT_PUSH_REG_5: u16 = 8;


fn main() {
    let filename = arg_file().expect("filename for signal expected as argument");
    let signal = fs::read_to_string(filename).expect("this should work");
    let signal: Vec<i32> = signal.split_whitespace().map(|num| num.parse().unwrap()).collect();
    let mut gps1 = ShiftRegGPS::new(0);
    println!("{:?}", gps1.get_code());
}

fn arg_file() -> Result<String, &'static str> {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("expected filename");
    }
    Ok(args.remove(1))
}

struct ShiftRegGPS {
    top: u16,
    bot: u16,
    id: u16,
}

impl ShiftRegGPS {
    #[inline]
    fn reg_size() -> u16 {
        10
    }

    fn new(id: u16) -> ShiftRegGPS {
        ShiftRegGPS { top: 0xffff, bot: 0xffff, id }
    }

    //inverse bit order
    fn get_bit(index: u16, reg: u16) -> u16 {
        if ((1 << (ShiftRegGPS::reg_size() - index)) & reg) == 0 {
            return 0;
        }
        return 1;
    }

    fn next_bit(&mut self) -> u16 {
        let out_top = self.top & 0x0001;
        let next_top =
                ShiftRegGPS::get_bit(TOP_PUSH_REG, self.top) ^
                out_top;

        let calc_bot =
                ShiftRegGPS::get_bit(GPS_REGISTER_PROPS[self.id as usize].0, self.bot) ^
                ShiftRegGPS::get_bit(GPS_REGISTER_PROPS[self.id as usize].1, self.bot);

        let out_bot = self.bot & 0x0001;
        let next_bot =
                ShiftRegGPS::get_bit(BOT_PUSH_REG_1, self.bot) ^
                ShiftRegGPS::get_bit(BOT_PUSH_REG_2, self.bot) ^
                ShiftRegGPS::get_bit(BOT_PUSH_REG_3, self.bot) ^
                ShiftRegGPS::get_bit(BOT_PUSH_REG_4, self.bot) ^
                ShiftRegGPS::get_bit(BOT_PUSH_REG_5, self.bot) ^
                out_bot;
        //update state
        self.top ^= 1 << ShiftRegGPS::reg_size();
        self.bot ^= 1 << ShiftRegGPS::reg_size();
        self.top |= next_top << ShiftRegGPS::reg_size();
        self.bot |= next_bot << ShiftRegGPS::reg_size();
        out_top ^ calc_bot
    }

    fn get_code(&mut self) -> Vec<u16> {
        let mut vec = Vec::new();
        for _ in 0..CODE_LENGTH {
            vec.push(self.next_bit())
        }
        vec
    }
}

