use std::iter::zip;

use anyhow::Result;
use indoc::formatdoc;
use libloading::Symbol;
use testutil::{decoders::*, util::*};

#[test]
fn message_id_lookup() -> Result<()> {
    // Make a CAN network with some messages
    let desc = formatdoc! {"
        nodes:
            TEST:
                messages:
                    Message1:
                        id: 0x10
                        signals:
                            testSignal:
                                width: 1
                    Message2:
                        id: 0x11
                        signals:
                            testSignal2:
                                width: 1
    "};

    let net = opencan_compose::compose_entry_str(&desc)?;

    // Do codegen
    let args = opencan_codegen::Args {
        node: "TEST".into(),
        in_file: "".into(),
    };
    let c_content = opencan_codegen::Codegen::new(args, &net).network_to_c()?;

    // Compile
    println!("{c_content}");
    let lib = c_string_to_so(c_content)?;

    // Look up symbols
    let msg1_decode: Symbol<DecodeFn> = unsafe { lib.get(b"CANRX_decode_TEST_Message1")? };
    let msg2_decode: Symbol<DecodeFn> = unsafe { lib.get(b"CANRX_decode_TEST_Message2")? };
    let lookup: Symbol<fn(u32) -> Option<DecodeFn>> = unsafe { lib.get(b"CANRX_id_to_decode_fn")? };

    assert_eq!(lookup(0x10), Some(*msg1_decode));
    assert_eq!(lookup(0x11), Some(*msg2_decode));
    assert_eq!(lookup(0x99), None);
    Ok(())
}

#[test]
fn basic_compare_decoders() -> Result<()> {
    let desc = formatdoc! {"
        nodes:
            TEST:
                messages:
                    TestMessage:
                        id: 0x10
                        signals:
                            testSignal:
                                start_bit: 1
                                width: 57
    "};
    let net = opencan_compose::compose_entry_str(&desc)?;
    let cantools = CantoolsDecoder::new(&net)?;
    let opencan = CodegenDecoder::new(&net, "TEST")?;

    for msg in net.iter_messages() {
        let data = &[0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA]; // adjust length
        let cantools_answer = cantools.decode_message(&msg.name, data)?;
        let codegen_answer = opencan.decode_message(&msg.name, data)?;

        assert_eq!(cantools_answer.len(), codegen_answer.len());

        for (ct_sig, cg_sig) in zip(cantools_answer, codegen_answer) {
            assert_eq!(ct_sig, cg_sig);
        }
    }

    Ok(())
}
