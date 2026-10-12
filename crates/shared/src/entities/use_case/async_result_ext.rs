use std::future::Future;

/// Extension trait para encadear operações assíncronas em um `Result` no estilo Railway Oriented Programming (ROP).
pub trait AsyncResultExt<T, E> {
    /// Encadeia uma operação assíncrona sobre o valor de sucesso de um `Result`.
    /// Se o `Result` for `Err`, a operação assíncrona não é executada e o erro é propagado.
    fn and_then_async<F, Fut, U>(self, op: F) -> impl Future<Output = Result<U, E>>
    where
        F: FnOnce(T) -> Fut,
        Fut: Future<Output = Result<U, E>>;

    /// Aplica uma transformação assíncrona sobre o valor de sucesso de um `Result`.
    /// Se o `Result` for `Err`, a operação assíncrona não é executada e o erro é propagado.
    fn map_async<F, Fut, U>(self, op: F) -> impl Future<Output = Result<U, E>>
    where
        F: FnOnce(T) -> Fut,
        Fut: Future<Output = U>;
}

impl<T, E> AsyncResultExt<T, E> for Result<T, E> {
    async fn and_then_async<F, Fut, U>(self, op: F) -> Result<U, E>
    where
        F: FnOnce(T) -> Fut,
        Fut: Future<Output = Result<U, E>>,
    {
        match self {
            Ok(val) => op(val).await,
            Err(err) => Err(err),
        }
    }

    async fn map_async<F, Fut, U>(self, op: F) -> Result<U, E>
    where
        F: FnOnce(T) -> Fut,
        Fut: Future<Output = U>,
    {
        match self {
            Ok(val) => Ok(op(val).await),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_and_then_async_ok() {
        let res: Result<i32, &str> = Ok(10);
        let result = res
            .and_then_async(|val| async move { Ok::<_, &str>(val * 2) })
            .await;
        assert_eq!(result, Ok(20));
    }

    #[tokio::test]
    async fn test_and_then_async_err_input() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        let executed = Arc::new(AtomicBool::new(false));
        let executed_clone = executed.clone();
        let res: Result<i32, &str> = Err("initial error");
        let result = res
            .and_then_async(|val| {
                let executed = executed_clone.clone();
                async move {
                    executed.store(true, Ordering::SeqCst);
                    Ok::<_, &str>(val * 2)
                }
            })
            .await;
        assert_eq!(result, Err("initial error"));
        assert!(!executed.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_and_then_async_err_output() {
        let res: Result<i32, &str> = Ok(10);
        let result = res
            .and_then_async(|_| async move { Err::<i32, &str>("async error") })
            .await;
        assert_eq!(result, Err("async error"));
    }

    #[tokio::test]
    async fn test_map_async_ok() {
        let res: Result<i32, &str> = Ok(10);
        let result = res
            .map_async(|val| async move { val * 3 })
            .await;
        assert_eq!(result, Ok(30));
    }

    #[tokio::test]
    async fn test_map_async_err() {
        let res: Result<i32, &str> = Err("error");
        let result = res
            .map_async(|val| async move { val * 3 })
            .await;
        assert_eq!(result, Err("error"));
    }
}
