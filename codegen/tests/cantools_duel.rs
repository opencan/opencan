use std::iter::zip;

use anyhow::{Context, Result};
use indoc::formatdoc;
use libloading::Symbol;
use testutil::decoders::*;

#[test]
fn message_id_lookup() -> Result<()> {
    // Make a CAN network with some messages
    let desc = formatdoc! {"
        nodes:
        - TESTTX:
            messages:
            - Message1:
                id: 0x10
                signals:
                  - testSignal:
                      width: 1
            - Message2:
                id: 0x11
                signals:
                  - testSignal2:
                      width: 1
        - TESTRX:
            rx:
              - TESTTX_Message1
              - TESTTX_Message2
    "};

    let net = opencan_compose::compose_str(&desc)?;

    // Do codegen
    let dec = CodegenDecoder::new(&net, "TESTRX")?;

    // Look up symbols
    let msg1_decode: Symbol<DecodeFn> = unsafe { dec.lib.get(b"CANRX_doRx_TESTTX_Message1")? };
    let msg2_decode: Symbol<DecodeFn> = unsafe { dec.lib.get(b"CANRX_doRx_TESTTX_Message2")? };
    let lookup: Symbol<fn(u32) -> Option<DecodeFn>> = unsafe { dec.lib.get(b"CANRX_id_to_rx_fn")? };

    assert_eq!(lookup(0x10), Some(*msg1_decode));
    assert_eq!(lookup(0x11), Some(*msg2_decode));
    assert_eq!(lookup(0x99), None);
    Ok(())
}

#[test]
fn basic_compare_decoders() -> Result<()> {
    let desc = include_str!("../../compose/gadgets/can.yml");

    let net = opencan_compose::compose_str(&desc)?;
    let cantools = CantoolsDecoder::new(&net)?;
    let opencan = CodegenDecoder::new(&net, "TEST")?;

    for node in net.iter_nodes() {
        eprintln!("---- Node: {}", node.name);
        for msg in net
            .tx_messages_by_node(&node.name)
            .context("Expected node to exist")?
        {
            eprintln!("------ message: {}", msg.name);

            // let data = &[0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA]; // adjust length
            let data = 18446744073709551557u64.to_ne_bytes(); // big boy prime number
            let data = &data[0..(msg.length as usize)];
            let cantools_answer = cantools.decode_message(&msg.name, data)?;
            let codegen_answer = opencan.decode_message(&msg.name, data)?;

            assert_eq!(cantools_answer.len(), codegen_answer.len());

            for (ct_sig, cg_sig) in zip(cantools_answer, codegen_answer) {
                assert_eq!(ct_sig, cg_sig);
                eprint!("cantools: {ct_sig:?}\ncodegen:  {cg_sig:?}\n\n");
            }
        }
    }

    // panic!();

    Ok(())
}
