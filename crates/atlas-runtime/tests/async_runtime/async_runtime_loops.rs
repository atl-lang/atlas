use super::*;

#[test]
fn test_channel_multiple_sends_before_receive() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        // Send many messages
        for i in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
            channelSend(sender, i);
        }

        // Receive first message
        await channelReceive(receiver)
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_channel_interleaved_send_receive() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        // Send 5 values then receive 5 values, verify receive works for each
        channelSend(sender, 0);
        channelSend(sender, 1);
        channelSend(sender, 2);
        channelSend(sender, 3);
        channelSend(sender, 4);

        let v0 = await channelReceive(receiver);
        let v1 = await channelReceive(receiver);
        let v2 = await channelReceive(receiver);
        let v3 = await channelReceive(receiver);
        let v4 = await channelReceive(receiver);

        // Verify first and last values
        v0 == 0 && v4 == 4
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Bool(true)); // receive works for all 5
}

#[test]
#[ignore = "requires async VM: async-block syntax and blocking task join not yet implemented"]
fn test_complex_async_workflow() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        // Producer task
        spawn(async {
            for i in [1, 2, 3, 4, 5] {
                await sleep(2);
                channelSend(sender, i);
            }
        }, "producer");

        // Consumer task
        let mut sum: number = 0;
        for i in [0, 1, 2, 3, 4] {
            let val = await channelReceive(receiver);
            sum = sum + val;
            let _unused = i;
        }

        sum
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(15.0)); // 1+2+3+4+5
}
