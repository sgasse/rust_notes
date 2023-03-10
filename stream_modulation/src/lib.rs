use async_stream::stream;
use futures::{Stream, StreamExt};
use tokio::time::Duration;

/// Throttle a stream.
///
/// The throttled stream will send updates only at regular intervals.
/// Let's consider the following test sequence with a throttle delay of 500ms:
/// `(0ms, 1), (200ms, 2), (400ms, 3)`
/// The throttled stream will let the first element immediately pass, wait out
/// the second element and yield the third after the throttle delay has elapsed.
/// The output sequence is thus `(0ms, 1), (500ms, 3)`.
pub fn throttle_stream<St>(
    mut stream: St,
    throttle_delay: Duration,
) -> Box<dyn Stream<Item = St::Item> + Send + Sync + 'static + Unpin>
where
    St: Stream + Send + Sync + 'static + Unpin,
    St::Item: Send + Sync + std::fmt::Debug,
{
    let mut timer = tokio::time::interval(throttle_delay);
    let mut delay_elapsed = false;
    let mut data: Option<St::Item> = None;

    let stream = Box::pin(stream! {
        loop {
            tokio::select!{
                res = stream.next() => match res {
                    None => break,
                    Some(item) => {
                        if delay_elapsed {
                            delay_elapsed = false;
                            data = None;
                            yield item;
                        } else {
                            data.replace(item);
                        }
                    }
                },
                _ = timer.tick() => {
                    if let Some(item) = data.take() {
                        yield item;
                    } else {
                        delay_elapsed = true;
                    }
                }
            }
        }

    });

    Box::new(stream)
        as Box<(dyn Stream<Item = <St as Stream>::Item> + Send + Sync + 'static + Unpin)>
}

/// Debounce a stream.
///
/// The debounced stream will only yield an update after no new updates have
/// been sent for the debounce delay. Let's consider the test sequence below
/// with a debounce delay of 500ms:
/// `(0ms, 1), (200ms, 2), (400ms, 3)`
/// For the first two updates, the stream will store the updates to send them
/// at 500ms/700ms but they are replaced by the last one, which is set to be
/// sent at 900ms (500ms after the last update). The output sequence is thus:
/// `(900ms, 3)`
pub fn debounce_stream<St>(
    mut stream: St,
    debounce_delay: Duration,
) -> Box<dyn Stream<Item = St::Item> + Send + Sync + 'static + Unpin>
where
    St: Stream + Send + Sync + 'static + Unpin,
    St::Item: Send + Sync + std::fmt::Debug,
{
    let mut timer = tokio::time::interval(debounce_delay);
    let mut data: Option<St::Item> = None;

    let stream = Box::pin(stream! {
        loop {
            tokio::select!{
                res = stream.next() => match res {
                    None => break,
                    Some(item) => {
                        data.replace(item);
                        timer.reset();
                    }
                },
                _ = timer.tick() => {
                    if let Some(item) = data.take() {
                        yield item;
                    }
                }
            }
        }

    });

    Box::new(stream)
        as Box<(dyn Stream<Item = <St as Stream>::Item> + Send + Sync + 'static + Unpin)>
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::time::Instant;

    fn example_handcrafted() -> Box<dyn Stream<Item = i32> + Send + Sync + 'static + Unpin> {
        let stream = Box::pin(stream! {
            yield 1;
            tokio::time::sleep(Duration::from_millis(200)).await;
            yield 2;
            tokio::time::sleep(Duration::from_millis(200)).await;
            yield 3;
            tokio::time::sleep(Duration::from_millis(600)).await;
        });
        Box::new(stream) as Box<dyn Stream<Item = i32> + Send + Sync + 'static + Unpin>
    }

    #[tokio::test]
    async fn test_throttle() {
        let stream =
            example_handcrafted() as Box<dyn Stream<Item = i32> + Send + Sync + 'static + Unpin>;
        let mut throttled_stream = throttle_stream(stream, Duration::from_millis(500));

        let mut res = Vec::with_capacity(3);
        let start = Instant::now();
        while let Some(value) = throttled_stream.next().await {
            println!("Received {value} at {:?}", start.elapsed());
            res.push(value);
        }
        assert_eq!(res, vec![1, 3]);
    }

    #[tokio::test]
    async fn test_debounce() {
        let stream =
            example_handcrafted() as Box<dyn Stream<Item = i32> + Send + Sync + 'static + Unpin>;
        let mut throttled_stream = debounce_stream(stream, Duration::from_millis(500));

        let mut res = Vec::with_capacity(3);
        let start = Instant::now();
        while let Some(value) = throttled_stream.next().await {
            println!("Received {value} at {:?}", start.elapsed());
            res.push(value);
        }
        assert_eq!(res, vec![3]);
    }
}
