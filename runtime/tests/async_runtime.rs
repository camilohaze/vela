//! Tests for async runtime

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    use std::future::Future;
    use std::pin::Pin;
    use vela_runtime::r#async::{
        AsyncExecutor, VelaFuture, Promise, Task, AsyncError, AsyncResult, utils
    };

    // Note: These tests use Tokio's test runtime instead of creating their own
    // AsyncExecutor instances, as Tokio doesn't allow nested runtimes

    #[tokio::test]
    async fn test_promise_resolve() {
        let (sender, promise) = Promise::<i32>::new();

        sender.resolve(123);

        let result = promise.await_promise().await.unwrap();
        assert_eq!(result, 123);
    }

    #[tokio::test]
    async fn test_promise_reject() {
        let (sender, promise) = Promise::<i32>::new();

        sender.reject(AsyncError::TaskPanic("test error".to_string()));

        let result = promise.await_promise().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_utils_with_timeout_success() {
        let future = async {
            sleep(Duration::from_millis(10)).await;
            42
        };

        let result = utils::with_timeout(future, Duration::from_millis(100)).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_utils_with_timeout_expired() {
        let future = async {
            sleep(Duration::from_millis(100)).await;
            42
        };

        let result = utils::with_timeout(future, Duration::from_millis(10)).await;
        assert!(matches!(result, Err(AsyncError::Timeout)));
    }

    #[tokio::test]
    async fn test_utils_select_first() {
        let futures = vec![
            Box::pin(async {
                sleep(Duration::from_millis(50)).await;
                "slow"
            }) as Pin<Box<dyn Future<Output = &str> + Send>>,
            Box::pin(async {
                sleep(Duration::from_millis(10)).await;
                "fast"
            }),
        ];

        let result = utils::select_first(futures).await.unwrap();
        assert_eq!(result, "fast");
    }

    #[tokio::test]
    async fn test_utils_join_all() {
        let futures = vec![
            Box::pin(async { Ok(1) }) as Pin<Box<dyn Future<Output = AsyncResult<i32>> + Send>>,
            Box::pin(async { Ok(2) }),
            Box::pin(async { Ok(3) }),
        ];

        let results = utils::join_all(futures).await;
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].as_ref().unwrap(), &1);
        assert_eq!(results[1].as_ref().unwrap(), &2);
        assert_eq!(results[2].as_ref().unwrap(), &3);
    }

    // Tests that would require AsyncExecutor are commented out for now
    // They would need to be run in a separate process or with a different testing approach

    /*
    #[test]
    fn test_async_executor_creation() {
        let executor = AsyncExecutor::new();
        assert!(executor.is_ok());
    }

    #[test]
    fn test_async_executor_with_workers() {
        let executor = AsyncExecutor::with_workers(2);
        assert!(executor.is_ok());
    }

    #[tokio::test]
    async fn test_spawn_task() {
        let executor = AsyncExecutor::new().unwrap();

        let handle = executor.spawn(async {
            sleep(Duration::from_millis(10)).await;
            42
        });

        let result = executor.block_on(handle).unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_spawn_blocking() {
        let executor = AsyncExecutor::new().unwrap();

        let handle = executor.spawn_blocking(|| {
            std::thread::sleep(Duration::from_millis(10));
            100
        });

        let result = executor.block_on(handle).unwrap();
        assert_eq!(result, 100);
    }

    #[tokio::test]
    async fn test_vela_future() {
        let executor = AsyncExecutor::new().unwrap();

        let future = VelaFuture::new(async {
            sleep(Duration::from_millis(10)).await;
            "hello"
        });

        let result = future.await_blocking(&executor).unwrap();
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn test_task_spawn_and_await() {
        let executor = AsyncExecutor::new().unwrap();

        let task = Task::spawn(&executor, async {
            sleep(Duration::from_millis(10)).await;
            999
        });

        let result = task.await_task().await.unwrap();
        assert_eq!(result, 999);
    }
    */
}