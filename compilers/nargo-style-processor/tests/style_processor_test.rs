use nargo_style_processor::{PreprocessorType, StyleProcessor, minify, optimize, process, process_with_preprocessor, process_with_used_selectors, watch};

#[test]
fn test_process_style() {
    let mut processor = StyleProcessor::new();
    let css = ".test { color: red; }";
    let processed = processor.process(css, None).unwrap();

    assert!(processed.contains(".test { color: red; }"));
    assert!(processed.contains("/* Processed by Nargo Style Processor */"));
}

#[test]
fn test_scss_preprocessor() {
    let mut processor = StyleProcessor::new().with_preprocessor(PreprocessorType::Scss);
    let scss = "$primary-color: #4285f4; .test { color: $primary-color; }";
    let processed = processor.process(scss, None).unwrap();

    assert!(processed.contains(".test"));
    assert!(processed.contains("color: #4285f4"));
}

#[test]
fn test_less_preprocessor() {
    let mut processor = StyleProcessor::new().with_preprocessor(PreprocessorType::Less);
    let less = "@primary-color: #4285f4; .test { color: @primary-color; }";
    let processed = processor.process(less, None).unwrap();

    assert!(processed.contains(".test"));
    assert!(processed.contains("color: #4285f4"));
}

#[test]
fn test_stylus_preprocessor() {
    let mut processor = StyleProcessor::new().with_preprocessor(PreprocessorType::Stylus);
    let stylus = ".test\n  color: red";
    let processed = processor.process(stylus, None).unwrap();

    assert!(processed.contains(".test"));
    assert!(processed.contains("color: red"));
}

#[test]
fn test_minify() {
    let mut processor = StyleProcessor::new().with_minify(true);
    let css = ".test { color: red; }";
    let processed = processor.process(css, None).unwrap();

    assert!(!processed.contains("/* Processed by Nargo Style Processor */"));
    assert!(processed.contains(".test{color:red;}"));
}

#[test]
fn test_remove_unused() {
    let mut processor = StyleProcessor::new().with_remove_unused(true);
    let css = ".used { color: red; } .unused { color: blue; }";
    let used_selectors = vec![".used".to_string()];
    let processed = processor.process(css, Some(&used_selectors)).unwrap();

    assert!(processed.contains(".used"));
    assert!(!processed.contains(".unused"));
}

#[test]
fn test_watch_function() {
    // 创建临时测试文件
    use std::{fs::File, io::Write};

    // 创建测试文件
    let test_file = "test_watch.css";
    let output_file = "test_watch_output.css";

    // 写入测试内容
    let mut file = File::create(test_file).unwrap();
    writeln!(file, ".test {{ color: red; }}").unwrap();

    // 测试 watch 函数
    let processor = StyleProcessor::new();
    let result = watch(&processor, test_file, output_file, None);

    // 清理临时文件
    std::fs::remove_file(test_file).ok();
    std::fs::remove_file(output_file).ok();

    // 断言结果
    assert!(result.is_ok());
}

#[test]
fn test_process_function() {
    let css = ".test { color: red; }";
    let processed = process(css).unwrap();
    assert!(processed.contains(".test { color: red; }"));
}

#[test]
fn test_process_with_used_selectors_function() {
    let css = ".used { color: red; } .unused { color: blue; }";
    let used_selectors = vec![".used".to_string()];
    let processed = process_with_used_selectors(css, &used_selectors).unwrap();
    assert!(processed.contains(".used"));
    assert!(!processed.contains(".unused"));
}

#[test]
fn test_minify_function() {
    let css = ".test { color: red; }";
    let processed = minify(css).unwrap();
    assert!(!processed.contains("/* Processed by Nargo Style Processor */"));
    assert!(processed.contains(".test{color:red;}"));
}

#[test]
fn test_optimize_function() {
    let css = ".used { color: red; } .unused { color: blue; }";
    let used_selectors = vec![".used".to_string()];
    let processed = optimize(css, &used_selectors).unwrap();
    assert!(processed.contains(".used"));
    assert!(!processed.contains(".unused"));
    assert!(!processed.contains("/* Processed by Nargo Style Processor */"));
}

#[test]
fn test_process_with_preprocessor_function() {
    let scss = "$primary-color: #4285f4; .test { color: $primary-color; }";
    let processed = process_with_preprocessor(scss, PreprocessorType::Scss).unwrap();
    assert!(processed.contains(".test"));
    assert!(processed.contains("color: #4285f4"));
}

#[test]
fn test_cache_performance() {
    let mut processor = StyleProcessor::new().with_preprocessor(PreprocessorType::Scss);
    let scss = "$primary-color: #4285f4; .test { color: $primary-color; }";

    // 第一次处理，应该没有缓存
    let start1 = std::time::Instant::now();
    let processed1 = processor.process(scss, None).unwrap();
    let duration1 = start1.elapsed();

    // 第二次处理，应该使用缓存
    let start2 = std::time::Instant::now();
    let processed2 = processor.process(scss, None).unwrap();
    let duration2 = start2.elapsed();

    // 验证两次处理结果相同
    assert_eq!(processed1, processed2);
    // 验证第二次处理时间比第一次短（缓存生效）
    assert!(duration2 < duration1);
}
