use crossbeam_queue::ArrayQueue;
use alloc::string::String;
use x86_64::instructions::interrupts;

once_mutex!(pub INPUT_BUFFER: ArrayQueue<u8>);

/// push_key: Push a key to the input buffer after lock INPUT_BUFFER
pub fn push_key(key: u8) {
    if let Some(buffer) = get_buffer() {
        if buffer.is_full() { warn!("Input buffer is full, dropping key: {}", key); }
        else { buffer.push(key).expect("Input buffer push failed.");  }
    }
}

/// try_pop_key: Try to pop a key from the input buffer without stall
/// will return None if buffer is empty or lock failed
/// need to temperately disable interrupt to avoid deadlock
pub fn try_pop_key() -> Option<u8> {
    // 这里不能用x86_64::instructions::interrupts::disable();手动关中断，
    // 因为运行代码内容太多了，所以会导致互相卡住！！应该在最快的时间把锁释放掉

    // x86_64::instructions::interrupts::disable();
    // // trace!("try to get lock in try_pop_key");
    // let buffer = get_buffer();
    // if buffer.is_none() {
    //     x86_64::instructions::interrupts::enable();
    //     return None;
    // }
    // // trace!("Get lock in try_pop_key");
    // // trace!("{}", buffer.len());
    // let input_buffer = buffer.unwrap();
    // if input_buffer.is_empty() {
    //     x86_64::instructions::interrupts::enable();
    //     return None;
    // }
    // let key = input_buffer.pop();
    // x86_64::instructions::interrupts::enable();
    // return key;
    interrupts::without_interrupts(|| {get_buffer_for_sure().pop()})
}

/// pop_key: Pop a key from the input buffer use try_pop_key
/// will stall until buffer is not empty
pub fn pop_key() -> u8 {
    loop {
        let key = try_pop_key();
        if key != None {
            trace!("key: {}", key.unwrap());
            return key.unwrap();
        }
    }
}

/// get_line: Get a line from the input buffer till '\n'
/// will stall the buffer
pub fn get_line() -> String {
    let mut line = String::with_capacity(128);
    loop {
        // trace!("interrupts enable: {}", x86_64::instructions::interrupts::are_enabled());
        let key = pop_key();
        // trace!("key: {}", key);
        if key == 0xd as u8 {
            break;
        } else if key == 0x08 || key == 0x7F {
            if !line.is_empty() { line.pop(); }
            continue;    
        } 
        line.push(key as u8 as char);
    }
    line
}

pub fn init() {
    init_INPUT_BUFFER(ArrayQueue::<u8>::new(128));
    
    println!("[+] Input Buffer Initialized.");
}

guard_access_fn!(pub get_buffer(INPUT_BUFFER: ArrayQueue<u8>));