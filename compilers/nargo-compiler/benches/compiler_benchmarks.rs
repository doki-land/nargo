use criterion::{Criterion, criterion_group, criterion_main};
use nargo_compiler::{CompileOptions, Compiler};
use std::time::Duration;

/// 生成大型组件用于性能测试
fn generate_large_component(n: usize) -> String {
    let mut s = String::from("<template>\n  <div class=\"container\">\n");
    for i in 0..n {
        s.push_str(&format!("    <div class=\"item\" id=\"item-{}\">\n", i));
        s.push_str(&format!("      <span class=\"label\">Label {}:</span>\n", i));
        s.push_str(&format!("      <span class=\"value\">{{{{ item{} }}}}</span>\n", i));
        s.push_str(&format!("      <button @click=\"update{}\">Update</button>\n", i));
        s.push_str("    </div>\n");
    }
    s.push_str("  </div>\n</template>\n\n<script>\n");
    for i in 0..n {
        s.push_str(&format!("const [item{}, setItem{}] = createSignal({});\n", i, i, i));
        s.push_str(&format!("function update{}() {{ setItem{}(item{}() + 1); }}\n", i, i, i));
    }
    s.push_str("</script>\n\n<style scoped>\n.container { padding: 20px; }\n.item { margin: 10px; }\n.label { font-weight: bold; }\n</style>");
    s
}

/// 冷启动基准测试
fn cold_start_benchmark(c: &mut Criterion) {
    let source = generate_large_component(100);

    c.bench_function("cold_start", |b| {
        b.iter(|| {
            let mut compiler = Compiler::new();
            compiler.compile("Benchmark", &source).unwrap();
        });
    });
}

/// 增量构建基准测试（使用缓存）
fn incremental_build_benchmark(c: &mut Criterion) {
    let source = generate_large_component(100);
    let mut compiler = Compiler::new();

    compiler.compile("Benchmark", &source).unwrap();

    c.bench_function("incremental_build", |b| {
        b.iter(|| {
            compiler.compile("Benchmark", &source).unwrap();
        });
    });
}

/// 不同规模组件的编译基准测试
fn component_size_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_size");
    group.measurement_time(Duration::from_secs(10));

    let sizes = [10, 50, 100, 200];
    for &size in &sizes {
        let source = generate_large_component(size);
        let mut compiler = Compiler::new();

        group.bench_function(format!("size_{}", size), |b| {
            b.iter(|| {
                compiler.compile("Benchmark", &source).unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(50);
    targets = cold_start_benchmark, incremental_build_benchmark, component_size_benchmark
);
criterion_main!(benches);
