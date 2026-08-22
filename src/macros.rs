//! 内部宏。

/// 为「每个变体都实现了 `CmdExecutor` 的枚举」生成转发实现。
macro_rules! impl_cmd_executor {
    ($enum:ident { $($variant:ident),+ $(,)? }) => {
        impl $crate::CmdExecutor for $enum {
            async fn execute(self) -> ::anyhow::Result<()> {
                match self {
                    $(
                        $enum::$variant(inner) =>
                            $crate::CmdExecutor::execute(inner).await,
                    )+
                }
            }
        }
    };
}
