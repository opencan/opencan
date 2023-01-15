use std::iter::zip;

use anyhow::{Context, Result};
use indoc::formatdoc;
use libloading::Symbol;
use testutil::{decoders::*, util::*};

#[test]
fn message_id_lookup() -> Result<()> {
    // Make a CAN network with some messages
    let desc = formatdoc! {"
        nodes:
        - TEST:
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
    "};

    let net = opencan_compose::compose_str(&desc)?;

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
    // let desc = formatdoc! {"
    //     nodes:
    //         TEST:
    //             messages:
    //                 TestMessage:
    //                     id: 0x10
    //                     signals:
    //                         testSignal:
    //                             start_bit: 1
    //                             width: 57
    // "};
    // let net = opencan_compose::compose_entry_str(&desc)?;
    // let path = format!("{}/../../compose/gadgets/can.yml", std::env::var("CARGO_BIN_EXE_cantools_duel")?);
    // let path = "./compose/gadgets/can.yml".into();
    // let net = opencan_compose::compose_entry(opencan_compose::Args { in_file: path })?;

    let desc = formatdoc! {"
    ---

    # templategroups:
    #   dbwnodes:
    #     - BLINK
    #     - THROTTLE
    #     - BRAKE
    #     - ENCF
    #     - ENCR
    #     - PB
    #     - STEER

    nodes:
    - TEST:
        messages:
        - NodeStatus:
            id: 0xE0
            cycletime: 100

            signals:
              - sysStatus:
                  description: Status of the node.
                  enumerated_values:
                    - IDLE: auto
                    - UNHEALTHY: auto
                    - ACTIVE: auto
                    - ESTOP: auto

              - counter:
                  description: Counter for fun.
                  width: 8

              - resetReason:
                  description: Reset reason.
                  enumerated_values:
                    - POWERON: auto
                    - WATCHDOG_RESET: auto
                    - UNKNOWN: auto

              - esp32ResetReasonCode:
                  description: ESP32 reset reason code (enum RESET_REASON)
                  width: 5

        - NodeInfo:
            id: 0xD0
            cycletime: 1000

            signals:
              - gitHash:
                  description: Githash of the currently-running firmware.
                  width: 32

              - gitDirty:
                  description: Repository was dirty at build time.
                  width: 1

              - eepromIdentity:
                  description: EEPROM identity.
                  width: 6

    - THROTTLE:
        messages:
        - AccelData:
            id: 0x10
            cycletime: 100

            signals:
              - throttleACmd:
                  description: Throttle A command.
                  width: 8

              - throttleFCmd:
                  description: Throttle B command.
                  width: 8

              - percent:
                  description: Percentage commanded.
                  width: 8
                  unit: percent

              - relayState:
                  description: Current relay state.
                  width: 1

    - BRAKE:
        messages:
        - BrakeData:
              id: 0x11
              cycletime: 100

              signals:
                - frequency:
                    description: Frequency of PWM input.
                    width: 16
                    unit: hertz
                - resolution:
                    description: Resolution of PWM input.
                    width: 16
                - dutyCycle:
                    description: Duty Cycle of PWM input.
                    width: 16
                - percent:
                    description: Percentage commanded.
                    width: 8
                    unit: percent
                - pressure:
                    description: Pressure Sensed.
                    width: 8

    - ENCF:
        messages:
        - EncoderData:
            id: 0x13
            cycletime: 10

            signals:
              - encoderLeft:
                  description: Left encoder pulses since last reading.
                  # is_signed: true
                  width: 12

              - encoderRight:
                  description: Right encoder pulses since last reading.
                  # is_signed: true
                  width: 12

              - dtUs:
                  description: Microseconds since last reading.
                  width: 16

    - ENCR:
        messages:
        - EncoderData:
            id: 0x14
            cycletime: 10

            signals:
              - encoderLeft:
                  description: Left encoder pulses since last reading.
                  # is_signed: true
                  width: 12

              - encoderRight:
                  description: Right encoder pulses since last reading.
                  # is_signed: true
                  width: 12

              - dtUs:
                  description: Microseconds since last reading.
                  width: 16

    - PB:
        messages:
        - ParkingBrakeData:
            id: 0x15
            cycletime: 100

            signals:
              - pbSet:
                  description: State of the parking brake.
                  width: 1

              - magnetEnergized:
                  description: State of the parking brake magnet.
                  width: 1

              - armedESTOP:
                  description: State of the ESTOP arming.
                  width: 1

    - STEER:
        messages:
        - SteeringData:
            id: 0x16
            cycletime: 100

            signals:
              - angle:
                  description: Absolute steering angle in radians.
                  # minimum: -0.610865238
                  # maximum: 0.610865238
                  scale: 0.000000001
                  # is_signed: true
                  width: 32

              - encoderTimeoutSet:
                  description: Has the encoder timed out?
                  width: 1

              - oDriveConnected:
                  description: Is the ODrive connected?
                  width: 1

        - SteeringCmd:
            id: 0x21
            cycletime: 100

            signals:
              - angleCmd:
                  description: Absolute steering angle in radians.
                  # minimum: -0.471238898
                  # maximum: 0.471238898
                  scale: 0.000000001
                  # is_signed: true
                  width: 32

    - DBW:
        messages:
        - VelCmd:
            id: 0x20
            cycletime: 100

            signals:
              - throttlePercent:
                  description: Throttle percentage.
                  # maximum: 100
                  unit: percent
                  width: 7
              - brakePercent:
                  description: Brake percentage.
                  # maximum: 100
                  unit: percent
                  width: 7

        - ESTOP:
            id: 0x41F
            cycletime: 100

            signals:
              - src:
                  description: ESTOP source.
                  width: 8
                  enumerated_values:
                    - NODE: auto
                    - NODESD: auto
                    - SAFED: auto

              - reason:
                  description: ESTOP reason.
                  width: 8
                  enumerated_values:
                    - FAIL: auto
                    - TIMEOUT: auto
                    - INVALID_STATE: auto
                    - LIMIT_EXCEEDED: auto

        - Active:
            id: 0x420
            cycletime: 100

            signals:
              - active:
                  description: DBW is active.
                  width: 1

        - Enable:
            id: 0x421
            cycletime: 100

            signals:
              - enable:
                  description: Enable DBW active.
                  width: 1

    - WHL:
        messages:
        - AbsoluteEncoder:
            id: 0x1E5
            cycletime: 10

            signals:
              - foo:
                  description: Foo.
                  width: 8

                # TODO: add big endian support to OpenCAN
                #
                # this signal is a signed big-endian value
                # OpenCAN will decode this signal into an unsigned int
                # it is the job of the programmer to re-encode this value
                # into a signed int
              - encoder:
                  description: Absolute encoder position.
                  width: 16

              - bar:
                  description: Bar.
                  width: 40

      # - DBW_UpdaterUpdateTrigger:
      #     id: 0x4A0
      #     cycletime: 100

      #     template: dbwnodes

      #     signals:
      #       - trigger:
      #           description: Trigger the update.
      #           width: 1
      #       - filler1:
      #           description: Filler.
      #           width: 7
      #       - begin:
      #           description: Begin the update.
      #           width: 1

      # - DBW_UpdaterUpdateData:
      #     id: 0x4B0
      #     cycletime: 100

      #     template: dbwnodes

      #     signals:
      #       - position:
      #           description: Index of 32-bit data chunk.
      #           width: 24
      #       - data:
      #           description: Image data.
      #           width: 40

      # - DBW_NodeUpdateResponse:
      #     id: 0x4C0
      #     cycletime: 100

      #     template: dbwnodes

      #     signals:
      #       - ready:
      #           description: Ready to begin update.
      #           width: 1

      # - DBW_UpdaterUpdateDone:
      #     id: 0x4D0
      #     cycletime: 100

      #     template: dbwnodes

      #     signals:
      #       - finalSize:
      #           description: Number of bytes in final image size.
      #           width: 32

    "};

    let net = opencan_compose::compose_str(&desc)?;
    let cantools = CantoolsDecoder::new(&net)?;

    for node in net.iter_nodes() {
        eprintln!("---- Node: {}", node.name);
        let opencan = CodegenDecoder::new(&net, &node.name)?;
        for msg in net
            .messages_by_node(&node.name)
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
