
use embassy_time::{Duration, Timer};
use embassy_stm32::{
    i2c::I2c,
    mode::Blocking,
    i2c::Error,
    Peripherals,
    time::Hertz,
};

// LSM303AGR Registers
//const LSM303AGR_ACC_ADDR: u8 = 0x19;
const LSM303AGR_MAG_ADDR: u8 = 0x1E;

// Magnetometer registers
const CFG_REG_A_M: u8 = 0x60;
const CFG_REG_C_M: u8 = 0x62;
const OUTX_L_REG_M: u8 = 0x68;

pub struct Lsm303agr<'a> {
    i2c: I2c<'a, Blocking>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MagnetometerData {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl<'a> Lsm303agr<'a>
{
    pub fn new(per: Peripherals) -> Self {
        let i2c = I2c::new_blocking(
            per.I2C1,
            per.PB6,
            per.PB7,
            Hertz(100_000),
            Default::default()
        );
        Self {
            i2c,
        }
    }

    pub async fn init(&mut self) -> Result<(), Error> {

        self.i2c.blocking_write(LSM303AGR_MAG_ADDR, &[CFG_REG_A_M, 0x0C])?;
        self.i2c.blocking_write(LSM303AGR_MAG_ADDR, &[CFG_REG_C_M, 0x10])?;
        
        Timer::after(Duration::from_millis(10)).await;
        Ok(())
    }

    pub async fn read_magnetometer(&mut self) -> Result<MagnetometerData, Error> {
        let mut buffer = [0u8; 6];

        //self.i2c.blocking_write_read(LSM303AGR_MAG_ADDR, &[OUTX_L_REG_M | 0x80], &mut buffer)?;
        self.i2c.blocking_read(LSM303AGR_MAG_ADDR, &mut buffer)?;

        let x = i16::from_le_bytes([buffer[0], buffer[1]]);
        let y = i16::from_le_bytes([buffer[2], buffer[3]]);
        let z = i16::from_le_bytes([buffer[4], buffer[5]]);
    
        Ok(MagnetometerData { x, y, z })
    }

    pub fn convert_mag_to_gauss(raw: i16) -> f32 {
        // LSM303AGR magnetometer sensitivity is 1.5 mG/LSB
        raw as f32 * 0.0015
    }
}
