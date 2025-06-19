use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use database::{Document, Value, bson::{BsonEncoder, BsonDecoder}};
use std::collections::BTreeMap;
use std::io::Cursor;

fn create_sample_document(size: usize) -> Document {
    let mut doc = Document::new();
    for i in 0..size {
        doc.set(format!("field_{}", i), Value::I32(i as i32));
        doc.set(format!("string_{}", i), Value::String(format!("value_{}", i)));
        doc.set(format!("array_{}", i), Value::Array(vec![
            Value::I32(i as i32),
            Value::String(format!("arr_val_{}", i)),
            Value::F64(i as f64)
        ]));
    }
    doc
}

fn create_nested_document(depth: usize) -> Document {
    fn nested(current_depth: usize, max_depth: usize) -> BTreeMap<String, Value> {
        let mut map = BTreeMap::new();
        if current_depth < max_depth {
            map.insert("nested".to_string(), Value::Object(nested(current_depth + 1, max_depth)));
        } else {
            map.insert("value".to_string(), Value::I32(42));
        }
        map
    }
    let mut doc = Document::new();
    doc.set("root", Value::Object(nested(0, depth)));
    doc
}

fn create_mixed_document() -> Document {
    let mut doc = Document::new();
    
    // Add primitive values
    doc.set("null", Value::Null);
    doc.set("bool", Value::Bool(true));
    doc.set("int32", Value::I32(42));
    doc.set("int64", Value::I64(i64::MAX));
    doc.set("double", Value::F64(3.14159));
    doc.set("string", Value::String("Hello, BSON!".to_string()));
    
    // Add array with mixed types
    let array = vec![
        Value::I32(1),
        Value::String("array_string".to_string()),
        Value::Bool(false),
        Value::Null,
    ];
    doc.set("array", Value::Array(array));
    
    // Add nested document
    let mut nested_map = BTreeMap::new();
    nested_map.insert("nested_field".to_string(), Value::I32(99));
    nested_map.insert("nested_string".to_string(), Value::String("nested value".to_string()));
    doc.set("nested_doc", Value::Object(nested_map));
    
    doc
}

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    // Benchmark different document sizes
    for size in [10, 100, 1000, 10000].iter() {
        let doc = create_sample_document(*size);
        group.bench_with_input(BenchmarkId::new("serialize", size), &doc, |b, doc| {
            b.iter(|| {
                let mut buffer = Cursor::new(Vec::new());
                let mut encoder = BsonEncoder::new(&mut buffer);
                encoder.encode_document(black_box(doc))
            })
        });
    }

    // Benchmark nested documents
    for depth in [5, 10, 20, 50].iter() {
        let doc = create_nested_document(*depth);
        group.bench_with_input(BenchmarkId::new("serialize_nested", depth), &doc, |b, doc| {
            b.iter(|| {
                let mut buffer = Cursor::new(Vec::new());
                let mut encoder = BsonEncoder::new(&mut buffer);
                encoder.encode_document(black_box(doc))
            })
        });
    }

    // Benchmark mixed document
    let doc = create_mixed_document();
    group.bench_function("serialize_mixed", |b| {
        b.iter(|| {
            let mut buffer = Cursor::new(Vec::new());
            let mut encoder = BsonEncoder::new(&mut buffer);
            encoder.encode_document(black_box(&doc))
        })
    });

    group.finish();
}

fn bench_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialization");
    
    // Benchmark different document sizes
    for size in [10, 100, 1000, 10000].iter() {
        let doc = create_sample_document(*size);
        let mut buffer = Cursor::new(Vec::new());
        let mut encoder = BsonEncoder::new(&mut buffer);
        encoder.encode_document(&doc).unwrap();
        let bytes = buffer.into_inner();
        
        group.bench_with_input(BenchmarkId::new("deserialize", size), &bytes, |b, bytes| {
            b.iter(|| {
                let mut decoder = BsonDecoder::new(Cursor::new(bytes));
                decoder.decode_document()
            })
        });
    }

    // Benchmark nested documents
    for depth in [5, 10, 20, 50].iter() {
        let doc = create_nested_document(*depth);
        let mut buffer = Cursor::new(Vec::new());
        let mut encoder = BsonEncoder::new(&mut buffer);
        encoder.encode_document(&doc).unwrap();
        let bytes = buffer.into_inner();
        
        group.bench_with_input(BenchmarkId::new("deserialize_nested", depth), &bytes, |b, bytes| {
            b.iter(|| {
                let mut decoder = BsonDecoder::new(Cursor::new(bytes));
                decoder.decode_document()
            })
        });
    }

    // Benchmark mixed document
    let doc = create_mixed_document();
    let mut buffer = Cursor::new(Vec::new());
    let mut encoder = BsonEncoder::new(&mut buffer);
    encoder.encode_document(&doc).unwrap();
    let bytes = buffer.into_inner();
    
    group.bench_function("deserialize_mixed", |b| {
        b.iter(|| {
            let mut decoder = BsonDecoder::new(Cursor::new(&bytes));
            decoder.decode_document()
        })
    });

    group.finish();
}

fn bench_partial_document(c: &mut Criterion) {
    let mut group = c.benchmark_group("partial_operations");
    
    // Test with different document sizes
    for size in [100, 1000, 10000].iter() {
        let doc = create_sample_document(*size);
        let mut buffer = Cursor::new(Vec::new());
        let mut encoder = BsonEncoder::new(&mut buffer);
        encoder.encode_document(&doc).unwrap();
        let bytes = buffer.into_inner();
        
        // Test different numbers of fields
        for field_count in [1, 10, 50].iter() {
            let fields: Vec<String> = (0..*field_count)
                .map(|i| format!("field_{}", i * size / field_count))
                .collect();
            let fields_ref: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();
            
            group.bench_with_input(
                BenchmarkId::new(format!("partial_decode_{}fields", field_count), size),
                &(bytes.clone(), fields_ref.clone()),
                |b, (bytes, fields)| {
                    b.iter(|| {
                        let mut decoder = BsonDecoder::new(Cursor::new(bytes));
                        decoder.decode_partial_document(black_box(fields))
                    })
                }
            );
        }
    }

    group.finish();
}

fn bench_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming");
    
    // Test with different document sizes
    for size in [1000, 10000, 100000].iter() {
        let doc = create_sample_document(*size);
        let mut buffer = Cursor::new(Vec::new());
        let mut encoder = BsonEncoder::new(&mut buffer);
        encoder.encode_document(&doc).unwrap();
        let bytes = buffer.into_inner();
        
        group.bench_with_input(BenchmarkId::new("stream_encode", size), &doc, |b, doc| {
            b.iter(|| {
                let mut buffer = Cursor::new(Vec::new());
                let mut encoder = BsonEncoder::new(&mut buffer);
                encoder.encode_document(black_box(doc))
            })
        });

        group.bench_with_input(BenchmarkId::new("stream_decode", size), &bytes, |b, bytes| {
            b.iter(|| {
                let mut decoder = BsonDecoder::new(Cursor::new(bytes));
                decoder.decode_document()
            })
        });
    }

    // Test streaming multiple documents
    for doc_count in [10, 100, 1000].iter() {
        let doc = create_sample_document(100);
        let mut buffer = Cursor::new(Vec::new());
        let mut encoder = BsonEncoder::new(&mut buffer);
        
        // Create a stream of multiple documents
        for _ in 0..*doc_count {
            encoder.encode_document(&doc).unwrap();
        }
        let bytes = buffer.into_inner();
        
        group.bench_with_input(BenchmarkId::new("stream_multiple", doc_count), &bytes, |b, bytes| {
            b.iter(|| {
                let mut decoder = BsonDecoder::new(Cursor::new(bytes));
                decoder.decode_documents().collect::<Result<Vec<_>, _>>()
            })
        });
    }

    group.finish();
}

fn bench_field_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("field_extraction");
    
    // Test with different document sizes
    for size in [100, 1000, 10000].iter() {
        let doc = create_sample_document(*size);
        let mut buffer = Cursor::new(Vec::new());
        let mut encoder = BsonEncoder::new(&mut buffer);
        encoder.encode_document(&doc).unwrap();
        let bytes = buffer.into_inner();
        
        group.bench_with_input(BenchmarkId::new("get_field_names", size), &bytes, |b, bytes| {
            b.iter(|| {
                let mut decoder = BsonDecoder::new(Cursor::new(bytes));
                decoder.get_field_names()
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_serialization,
    bench_deserialization,
    bench_partial_document,
    bench_streaming,
    bench_field_extraction
);
criterion_main!(benches); 