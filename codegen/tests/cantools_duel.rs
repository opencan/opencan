use std::iter::zip;

use anyhow::Result;
use indoc::formatdoc;
use libloading::Symbol;
use opencan_core::{CANMessage, CANNetwork};
use testutil::{decoders::*, util::*};

// #[test]
fn message_id_lookup() -> Result<()> {
    // Make a CAN network with some messages
    let mut net = CANNetwork::new();
    let node = "TEST";
    net.add_node(node)?;

    let mut make_msg = |name: &str, id: u32| -> Result<CANMessage> {
        let msg = CANMessage::builder()
            .name(format!("{node}_{name}"))
            .id(id)
            .tx_node(node)
            .build()?;

        net.insert_msg(msg.clone())?;

        Ok(msg)
    };

    let msg1 = make_msg("Message1", 0x30)?;
    let msg2 = make_msg("Message2", 0x27)?;

    // Do codegen
    let args = opencan_codegen::Args {
        node: node.into(),
        in_file: "".into(),
    };
    let c_content = opencan_codegen::Codegen::new(args, &net).network_to_c()?;

    // Compile
    println!("{c_content}");
    let lib = c_string_to_so(c_content)?;

    // Look up symbols
    let decode_fn_name = |msg: &CANMessage| format!("CANRX_decode_{}", msg.name);

    let msg1_decode: Symbol<DecodeFn> = unsafe { lib.get(decode_fn_name(&msg1).as_bytes())? };
    let msg2_decode: Symbol<DecodeFn> = unsafe { lib.get(decode_fn_name(&msg2).as_bytes())? };
    let lookup: Symbol<fn(u32) -> Option<DecodeFn>> = unsafe { lib.get(b"CANRX_id_to_decode_fn")? };

    assert_eq!(lookup(msg1.id), Some(*msg1_decode));
    assert_eq!(lookup(msg2.id), Some(*msg2_decode));
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
