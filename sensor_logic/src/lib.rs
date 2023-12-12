
// A structure to track the usage of functions
struct CallsFunct {
    init_sensor_timer: bool,
    init_spi: bool,
    init_twi: bool,
    update_value: bool,
    configure_sensor: bool,
}

fn config_sensor(cf: &mut CallsFunct) {
    cf.init_sensor_timer = true;
    cf.configure_sensor = true;
    cf.init_spi = true;
}


pub fn add(left: u8, right: u8) -> u8 {
    left + right
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn calls_init(){
        let mut cf = CallsFunct{
            init_sensor_timer: false,
            init_spi: false,
            init_twi: false,
            update_value: false,
            configure_sensor: false,
        };

        config_sensor(&mut cf);

        assert_eq!(cf.configure_sensor, true);
        assert_eq!(cf.init_spi, true);
        assert_eq!(cf.init_sensor_timer, true);
    }
}
