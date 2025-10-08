use std::time::Duration;

pub fn do_after_delay<F>(f: F, delay: Duration)
where
    F: Fn() + Send + 'static,
{
    tokio::task::spawn(async move {
        tokio::time::sleep(delay).await;
        f();
    });
}
