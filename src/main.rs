use std::ffi::OsStr;
use std::{io, mem, ptr};
use sysinfo::{Process, System};

use mach2::{
    kern_return::{kern_return_t, KERN_SUCCESS},
    mach_types::{task_t, thread_act_array_t, thread_act_t},
    message::mach_msg_type_number_t,
    port::mach_port_name_t,
    task::{task_resume, task_suspend, task_threads},
    thread_act::{thread_get_state, thread_set_state},
    traps::{mach_task_self, task_for_pid},
    vm::mach_vm_read,
    vm_types::vm_offset_t,
};

use mach2::structs::arm_thread_state64_t as thread_state64_t;
use mach2::thread_status::ARM_THREAD_STATE64 as THREAD_STATE64;

use std::io::prelude::*;

fn resume(task: task_t) {
    unsafe {
        let kret = task_resume(task);
        if kret != KERN_SUCCESS {
            println!("Did not succeed in resuming task: {}", kret);
            panic!();
        }
    }
}

fn main() {
    // Step 0: Get process ID of target process
    print!("Enter process name: ");
    io::stdout().flush().ok();
    let mut line = String::new();

    io::stdin().read_line(&mut line).ok();

    let sys = System::new_all();
    let process: &Process = sys
        .processes_by_name(OsStr::new(line.trim_end()))
        .next()
        .unwrap();
    let pid = process.pid().as_u32() as i32;

    println!("Found process pid {}, attempting to attach", &pid);

    // Step 1: attach to the target process
    let task: mach_port_name_t = 0;
    let mut kret: kern_return_t;
    unsafe {
        kret = task_for_pid(
            mach_task_self() as mach_port_name_t,
            pid,
            mem::transmute(&task),
        );
    }

    if kret != KERN_SUCCESS {
        println!("Failed to attach (try sudo?): {}", kret);
        return;
    }
    println!("Successfully attached...");

    // Step 2: suspend execution of target process
    unsafe {
        kret = task_suspend(task as task_t);
    }
    if kret != KERN_SUCCESS {
        println!("Did not succeed in suspending task: {}", kret);
        return;
    }
    println!("Successfully suspended execution...");

    // Step 3: list threads of target process
    let thread_list: thread_act_array_t = ptr::null_mut();
    let thread_count: mach_msg_type_number_t = 0;
    unsafe {
        kret = task_threads(
            task as task_t,
            mem::transmute(&thread_list),
            mem::transmute(&thread_count),
        );
    }

    if kret != KERN_SUCCESS {
        println!("Did not succeed in getting task's threads: {}", kret);
        resume(task as task_t);
        return;
    }
    if thread_count != 1 {
        println!("Only single threaded programs supported");
        resume(task as task_t);
        return;
    }
    println!("Successfully found single thread...");

    // Step 4: get the initial state
    let mut state = thread_state64_t::new();
    let state_count = thread_state64_t::count();
    let thread: thread_act_t;
    unsafe {
        let threads = std::slice::from_raw_parts(thread_list, thread_count as usize);

        thread = *threads.iter().next().unwrap();

        kret = thread_get_state(
            thread,
            THREAD_STATE64,
            mem::transmute(&state),
            mem::transmute(&state_count),
        );
    }

    if kret != KERN_SUCCESS {
        println!("Did not succeed in getting thread state: {}", kret);
        resume(task as task_t);
        return;
    }
    println!("Successfully retrieved thread state:");

    // Print state of key registers
    println!("  FP: {:#016x}", state.__fp);
    println!("  LR: {:#016x}", state.__lr);
    println!("  SP: {:#016x}", state.__sp);
    println!("  PC: {:#016x}", state.__pc);

    // Step 5: escape from the loop
    print!("Choose option: 1 - instruction skip, 2 - unwind: ");
    io::stdout().flush().ok();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();

    // ********************
    if choice.starts_with('1') {
        // Strategy 1: skip an instruction
        state.__pc += 4;
        unsafe {
            kret = thread_set_state(thread, THREAD_STATE64, mem::transmute(&state), state_count);
            if kret != KERN_SUCCESS {
                println!("Did not succeed in setting state: {}", kret);
                resume(task as task_t);
                return;
            }
        }
    } else if choice.starts_with('2') {
        // Strategy 2: unwind the stack by one frame
        let read_count: mach_msg_type_number_t = 0;
        let offset: vm_offset_t = 0;
        unsafe {
            // Read 16 bytes from stack starting at the frame pointer
            // First 8 are the previous saved frame pointer
            // Second 8 are the previous saved link register (return address)
            kret = mach_vm_read(
                task,
                state.__fp,
                16,
                mem::transmute(&offset),
                mem::transmute(&read_count),
            );
            if kret != KERN_SUCCESS {
                println!("Did not succeed in reading memory: {}", kret);
                resume(task as task_t);
                return;
            }

            let stack_values: &[u64] = std::slice::from_raw_parts(offset as *const u64, 2);

            // Restore everything
            // Stack pointer in prior frame is current frame pointer, + 16 bytes
            // for the two saved registers
            state.__sp = state.__fp + 16;
            // Frame pointer and link register are restored from values found on the stack
            state.__fp = stack_values[0];
            state.__lr = stack_values[1];
            // Program counter set to same as link register (return address)
            state.__pc = state.__lr;
            kret = thread_set_state(thread, THREAD_STATE64, mem::transmute(&state), state_count);
        }

        if kret != KERN_SUCCESS {
            println!("Did not succeed in setting state: {}", kret);
            resume(task as task_t);
            return;
        }
    } else {
        println!("Unrecognized choice, no changes made");
    }
    // ********************
    println!("Done with updates, resuming execution");

    resume(task as task_t);
    println!("Successfully resumed process!");
}
