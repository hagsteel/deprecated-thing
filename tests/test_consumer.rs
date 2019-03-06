use sonr::prelude::*;
use sonr::reactor::producers::{ReactiveGenerator, ReactiveConsumer};
use sonr::errors::Result;

#[test]
fn test_consumer() -> Result<()> {
    let handle = System::init()?;

    let gen = ReactiveGenerator::new(vec![1, 2, 3, 4])?.map(|n| {
        eprintln!("orig: {:?}", n);
        n
    });

    let multiplier = ReactiveConsumer::new()?.map(|n| {
        eprintln!("con {:?}", n * 2);
        n * 2
    });

    let printer = ReactiveConsumer::new()?.map(|n| {
        if n == 8 {
            handle.send(SystemEvent::Stop);
        }
    });

    //let run = gen.chain(multiplier.chain(printer.noop()));
    let run = gen.chain(multiplier.chain(printer));

    System::start(run);
    Ok(())
}
