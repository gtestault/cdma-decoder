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

const TOP_PUSH_REG: u16 = 3;
const BOT_PUSH_REG_1: u16 = 2;
const BOT_PUSH_REG_2: u16 = 3;
const BOT_PUSH_REG_3: u16 = 6;
const BOT_PUSH_REG_4: u16 = 8;
const BOT_PUSH_REG_5: u16 = 9;


fn main() {
    let filename = arg_file().expect("filename for signal expected as argument");
    let signal = fs::read_to_string(filename).expect("this should work");
    let signal: Vec<i32> = signal.split_whitespace().map(|num| num.parse().unwrap()).collect();
    let mut gps1 = ShiftRegGPS::new(0);
    let gps_codes: Vec<Vec<u16>> =
        (0..24).map(|gps_id| {
            ShiftRegGPS::new(gps_id).get_code()
        }).collect();
    println!("{:?}", gps_codes[0]);
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

    fn push_bit_to_front(bit: u16, reg: &mut u16) {
        *reg >>= 1;
        *reg &= !(1u16 << ShiftRegGPS::reg_size() - 1);
        *reg |= bit << ShiftRegGPS::reg_size() - 1;
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
        ShiftRegGPS::push_bit_to_front(next_top, &mut self.top);
        ShiftRegGPS::push_bit_to_front(next_bot, &mut self.bot);
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_bit_test() {
        assert_eq!(1, ShiftRegGPS::get_bit(2, 0b0000_0001_0000_0000));
        assert_eq!(0, ShiftRegGPS::get_bit(2, 0b1111_1110_1111_1111));
        assert_eq!(1, ShiftRegGPS::get_bit(10, 0b0000_0000_0000_0001));
        assert_eq!(0, ShiftRegGPS::get_bit(10, 0b1111_1111_1111_1110));
    }

    #[test]
    fn push_bit_to_front_test() {
        let mut reg = 0b0000_0000_0000_0000;
        ShiftRegGPS::push_bit_to_front(1, &mut reg);
        assert_eq!(0b0000_0010_0000_0000, reg);
        ShiftRegGPS::push_bit_to_front(1, &mut reg);
        assert_eq!(0b0000_0011_0000_0000, reg);
        ShiftRegGPS::push_bit_to_front(0, &mut reg);
        println!("{}",format!("register value: {:b}", reg));
        assert_eq!(0b0000_0001_1000_0000, reg);
    }
}

