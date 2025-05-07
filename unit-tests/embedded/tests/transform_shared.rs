#![no_std]
#![no_main]

fn setup_log() {
    rtt_target::rtt_init_defmt!();
}

#[cfg(test)]
/// Need to explicitly state the executer
#[embedded_test::tests(setup=crate::setup_log())]
mod tests {
    use defmt::info;
    use embassy_stm32::Peripherals;
    use rtt_target as _;

    // An optional init function which is called before every test
    // Asyncness is optional, so is the return value
    #[init]
    async fn init() -> Peripherals {
        library::init_heap();
        let config = embassy_stm32::Config::default();
        let p = embassy_stm32::init(config);
        p
    }

    // Tests can be async (needs feature `embassy`)
    // Tests can take the state returned by the init function (optional)
    #[test]
    async fn takes_state(_state: Peripherals) {
        assert!(true)
    }

    // Tests can be ignored with the #[ignore] attribute
    #[test]
    #[ignore]
    fn it_works_ignored() {
        assert!(false)
    }

    // Tests can fail with a custom error message by returning a Result
    #[test]
    fn it_fails_with_err() -> Result<(), &'static str> {
        Err("It failed because ...")
    }

    // Tests can be annotated with #[should_panic] if they are expected to panic
    #[test]
    #[should_panic]
    fn it_passes() {
        info!("This is a log message from a test running on the target!");
        assert!(false)
    }

    // Tests can be annotated with #[timeout(<secs>)] to change the default timeout of 60s
    #[test]
    #[timeout(10)]
    fn it_timeouts() {
        loop {} // should run into the 10s timeout
    }

    #[test]
    fn transform_shared_type() {
        let mut shared = host::SharedType::new(1);
        info!("SharedType ID: {}", shared.id());
        host::transform_shared_type(&mut shared);
        info!("Transformed SharedType ID: {}", shared.id());

        assert_eq!(shared.id(), 2);
        assert_eq!(shared.array(), &[1, 2, 3, 4]);
    }
}
