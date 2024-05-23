
#[cfg(test)]
mod tests {
    use wgsl_preprocessor::*;

    #[test]
    fn test() {
        assert_eq!(preprocess("tests/test.wgsl").unwrap(), preprocess!("tests/test.wgsl"));
        assert_eq!(preprocess("tests/define.wgsl").unwrap(), preprocess!("tests/define.wgsl"));
        assert_eq!(preprocess("tests/comments.wgsl").unwrap(), preprocess!("tests/comments.wgsl"));
        assert_eq!(preprocess("tests/if.wgsl").unwrap(), preprocess!("tests/if.wgsl"));
        assert_eq!(preprocess("tests/args.wgsl").unwrap(), preprocess!("tests/args.wgsl"));

    }
}
