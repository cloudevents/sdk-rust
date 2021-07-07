#![no_std]
#![no_main]

use core::panic::PanicInfo;

use cloudevents;
use cloudevents::EventBuilder;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {
        #[allow(dead_code)]
        let event = cloudevents::EventBuilderV10::new()
            .id("my_id")
            .source("my_source")
            .subject("some_subject")
            .build()
            .unwrap();
    }
}
