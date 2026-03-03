use super::*;

#[test]
#[ignore = "requires tokio LocalSet context — re-enable when async runtime phase completes"]
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
#[ignore = "requires tokio LocalSet context — re-enable when async runtime phase completes"]
fn test_channel_interleaved_send_receive() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        let mut sum: number = 0;
        for i in [0, 1, 2, 3, 4] {
            channelSend(sender, i);
            let val = await channelReceive(receiver);
            sum = sum + val;
        }

        sum
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(10.0)); // 0+1+2+3+4 = 10
}

#[test]
#[ignore = "requires tokio LocalSet context — re-enable when async runtime phase completes"]
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
