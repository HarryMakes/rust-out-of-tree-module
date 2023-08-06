// SPDX-License-Identifier: GPL-2.0

//! Rust out-of-tree sample

// Here are some useful references for pin-init:
// (1) https://lwn.net/Articles/907876/ (LWN.net article about pin-init authors)
// (2) https://github.com/Rust-for-Linux/pinned-init
// (3) https://lore.kernel.org/lkml/D0mWM1KEcWLeFa7IIqPygHlXRTD6gRFHvJKaegYzQXo9zTx7YbSpVLeYLFfq53s2S30Wx7v0khkPMOy6Ng5HiNZ5x7TXtOyLB58vUHtq6ro=@protonmail.com/
//     (pin-init v1)
// (4) https://lwn.net/Articles/907876/ (pin-init v2)
// (5) https://lwn.net/Articles/927865/ (pin-init v3)
// (6) https://lwn.net/Articles/938615/ (pin-init QoL)

use core::ptr::NonNull;

use kernel::prelude::*;

module! {
    type: RustOutOfTree,
    name: "rust_out_of_tree",
    author: "Rust for Linux Contributors",
    description: "Rust out-of-tree sample",
    license: "GPL",
}

struct RustOutOfTree {
    numbers: Vec<i32>,
    rust_oft_pinned_data: Pin<Box<RustOutOfTreePinnedData>>,
    rust_oft_unpinned_data: RustOutOfTreeUnpinnedData,
}

#[pin_data]
struct RustOutOfTreePinnedData {
    rusty_number: MyRustyNumberStruct,
}

struct RustOutOfTreeUnpinnedData {
    rusty_number: MyRustyNumberStruct,
}

struct MyRustyNumberStruct {
    number: i32,
}

impl RustOutOfTreePinnedData {
    /// This initialiser allows a smart-pointer to initialise such a pinned data.
    /// (Read pinned_init::InPlaceInit for further details.)
    fn new(number: i32) -> impl PinInit<Self> {
        pin_init!(Self {
            rusty_number: MyRustyNumberStruct { number },
        })
    }
}

impl RustOutOfTreeUnpinnedData {
    fn new(number: i32) -> Self {
        Self {
            rusty_number: MyRustyNumberStruct { number },
        }
    }
}

impl MyRustyNumberStruct {
    fn set(&mut self, number: i32) {
        self.number = number;
        pr_info!("Updated Rusty Number @ {:p} to {}\n", self, self.number);
    }
}

impl kernel::Module for RustOutOfTree {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust out-of-tree sample (init)\n");

        let mut numbers = Vec::new();
        numbers.try_push(72)?;
        numbers.try_push(108)?;
        numbers.try_push(200)?;

        let rust_oft_pinned_data = Box::pin_init(RustOutOfTreePinnedData::new(2023))?;
        let rust_oft_unpinned_data = RustOutOfTreeUnpinnedData::new(3202);

        // Perform a test on pinned variables.
        // (Ref: https://doc.rust-lang.org/nightly/core/pin/index.html#example-self-referential-struct)
        let rusty_number_ptr = NonNull::from(&rust_oft_pinned_data.rusty_number);
        let rusty_number_value = unsafe { (*rusty_number_ptr.as_ptr()).number };
        let mut rust_oft_pinned_data_moved = rust_oft_pinned_data;
        let rusty_number_moved_ptr = NonNull::from(&rust_oft_pinned_data_moved.rusty_number);

        if rusty_number_ptr != rusty_number_moved_ptr {
            pr_err!(
                "FAIL Impossible! rust_oft_pinned_data.rusty_number {} is not pinned!\n",
                rusty_number_value
            );
        } else {
            pr_info!(
                "PASS! rust_oft_pinned_data.rusty_number {} @{:p} is pinned!\n",
                rusty_number_value,
                rusty_number_ptr.as_ptr()
            );
        }
        pr_info!(
            "rust_oft_pinned_data.rusty_number @{:p} is initially: {}\n",
            rusty_number_moved_ptr,
            rust_oft_pinned_data_moved.rusty_number.number
        );
        rust_oft_pinned_data_moved.rusty_number.set(20232023);

        // Perform a test on unpinned variables.
        let unpinned_rusty_number_ptr = NonNull::from(&rust_oft_unpinned_data.rusty_number);
        let unpinned_rusty_number_value = unsafe { (*unpinned_rusty_number_ptr.as_ptr()).number };
        let mut rust_oft_unpinned_data_moved = rust_oft_unpinned_data;
        let unpinned_rusty_number_moved_ptr =
            NonNull::from(&rust_oft_unpinned_data_moved.rusty_number);

        if unpinned_rusty_number_ptr == unpinned_rusty_number_moved_ptr {
            pr_err!("FAIL Impossible! rust_oft_unpinned_data.rusty_number {} is not moved to unpinned_rusty_number_moved!\n", unpinned_rusty_number_value);
        } else {
            pr_info!("PASS! rust_oft_unpinned_data.rusty_number {} @{:p} is moved to unpinned_rusty_number_moved @{:p}!\n", unpinned_rusty_number_value,
                     unpinned_rusty_number_ptr.as_ptr(), unpinned_rusty_number_moved_ptr.as_ptr());
        }
        pr_info!(
            "rust_oft_unpinned_data.rusty_number @{:p} is initially: {}\n",
            unpinned_rusty_number_moved_ptr,
            rust_oft_unpinned_data_moved.rusty_number.number
        );
        rust_oft_unpinned_data_moved.rusty_number.set(32023202);

        Ok(RustOutOfTree {
            numbers,
            rust_oft_pinned_data: rust_oft_pinned_data_moved,
            rust_oft_unpinned_data: rust_oft_unpinned_data_moved,
        })
    }
}

impl Drop for RustOutOfTree {
    fn drop(&mut self) {
        pr_info!("My numbers are {:?}\n", self.numbers);
        pr_info!(
            "My pinned number is {:?}\n",
            self.rust_oft_pinned_data.rusty_number.number
        );
        pr_info!(
            "My unpinned number is {:?}\n",
            self.rust_oft_unpinned_data.rusty_number.number
        );
        pr_info!("Rust out-of-tree sample (exit)\n");
    }
}
