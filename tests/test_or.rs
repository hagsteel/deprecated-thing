use sonr::errors::Result;
use sonr::prelude::*;
use sonr::reactor::producers::Mono;
use sonr::reactor::Either;
use sonr::reactor::consumers::Consume;

type R = Result<()>;


#[test]
fn test_or() -> R {
    let system_sig = System::init()?;

    let int_consumer = Consume::new();
    let string_consumer = Consume::new();

    // Convert the output to a u32 as `or` can only apply 
    // to reactors with the same `Output`.
    let string_consumer = string_consumer.map(|_| 0u32);

    let producer = Mono::new(2u32)?;
    let mut test_complete = false;
    let run = producer.map(|val: u32| {
        if val == 1 {
            Either::A(val)
        } else {
            Either::B(format!("{}", val))
        }
    }).chain(int_consumer.or(string_consumer).map(|_| {
        system_sig.send(SystemEvent::Stop);
        test_complete = true;
    }));

    System::start(run)?;

    assert!(test_complete);
    Ok(())
}
