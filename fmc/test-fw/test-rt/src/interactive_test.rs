// Licensed under the Apache-2.0 license

use caliptra_drivers::pcr_log::{PcrLogEntry, PcrLogEntryId};
use caliptra_drivers::{cprintln, PcrBank, PcrId, PersistentDataAccessor};
use caliptra_registers::pv::PvReg;
use ureg::RealMmioMut;

use core::convert::TryInto;
use zerocopy::AsBytes;

pub const TEST_CMD_READ_PCR_LOG: u32 = 0x1000_0000;
pub const TEST_CMD_READ_FHT: u32 = 0x1000_0001;
pub const TEST_CMD_READ_PCRS: u32 = 0x1000_0002;

fn process_mailbox_command(
    persistent_data: &PersistentDataAccessor,
    mbox: &caliptra_registers::mbox::RegisterBlock<RealMmioMut>,
) {
    let cmd = mbox.cmd().read();
    cprintln!("[fmc-test-harness] Received command: 0x{:08X}", cmd);
    match cmd {
        TEST_CMD_READ_PCR_LOG => {
            read_pcr_log(persistent_data, mbox);
        }
        TEST_CMD_READ_FHT => {
            read_fht(persistent_data, mbox);
        }
        TEST_CMD_READ_PCRS => {
            read_pcrs(mbox);
        }
        _ => {
            panic!();
        }
    }
}

pub fn process_mailbox_commands() {
    let persistent_data = unsafe { PersistentDataAccessor::new() };
    let mut mbox = unsafe { caliptra_registers::mbox::MboxCsr::new() };
    let mbox = mbox.regs_mut();

    cprintln!("Waiting for mailbox commands...");
    loop {
        if mbox.status().read().mbox_fsm_ps().mbox_execute_uc() {
            process_mailbox_command(&persistent_data, &mbox);
        }
    }
}

fn read_fht(
    persistent_data: &PersistentDataAccessor,
    mbox: &caliptra_registers::mbox::RegisterBlock<RealMmioMut>,
) {
    send_to_mailbox(mbox, persistent_data.get().fht.as_bytes(), true);
}

fn send_to_mailbox(
    mbox: &caliptra_registers::mbox::RegisterBlock<RealMmioMut>,
    data: &[u8],
    update_mb_state: bool,
) {
    let data_len = data.len();
    let word_size = core::mem::size_of::<u32>();
    let remainder = data_len % word_size;
    let n = data_len - remainder;
    for idx in (0..n).step_by(word_size) {
        mbox.datain()
            .write(|_| u32::from_le_bytes(data[idx..idx + word_size].try_into().unwrap()));
    }

    if remainder > 0 {
        let mut last_word = data[n] as u32;
        for idx in 1..remainder {
            last_word |= (data[n + idx] as u32) << (idx << 3);
        }
        mbox.datain().write(|_| last_word);
    }

    if update_mb_state {
        mbox.dlen().write(|_| data_len as u32);
        mbox.status().write(|w| w.status(|w| w.data_ready()));
    }
}

fn read_pcr_log(
    persistent_data: &PersistentDataAccessor,
    mbox: &caliptra_registers::mbox::RegisterBlock<RealMmioMut>,
) {
    let mut pcr_entry_count = 0;
    loop {
        let pcr_entry = persistent_data.get().pcr_log[pcr_entry_count];
        if PcrLogEntryId::from(pcr_entry.id) == PcrLogEntryId::Invalid {
            break;
        }

        pcr_entry_count += 1;
        send_to_mailbox(mbox, pcr_entry.as_bytes(), false);
    }

    mbox.dlen().write(|_| {
        (core::mem::size_of::<PcrLogEntry>() * pcr_entry_count)
            .try_into()
            .unwrap()
    });
    mbox.status().write(|w| w.status(|w| w.data_ready()));
}

fn read_pcrs(mbox: &caliptra_registers::mbox::RegisterBlock<RealMmioMut>) {
    let pcr_bank = unsafe { PcrBank::new(PvReg::new()) };
    const PCR_COUNT: usize = 32;
    for i in 0..PCR_COUNT {
        let pcr = pcr_bank.read_pcr(PcrId::try_from(i as u8).unwrap());
        let mut pcr_bytes: [u32; 12] = pcr.try_into().unwrap();

        swap_word_bytes_inplace(&mut pcr_bytes);
        send_to_mailbox(mbox, pcr.as_bytes(), false);
    }

    mbox.dlen().write(|_| (48 * PCR_COUNT).try_into().unwrap());
    mbox.status().write(|w| w.status(|w| w.data_ready()));
}

fn swap_word_bytes_inplace(words: &mut [u32]) {
    for word in words.iter_mut() {
        *word = word.swap_bytes()
    }
}
